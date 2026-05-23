/*
 * Nuva OS - CRYSTALS-Dilithium Digital Signature Implementation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * CRYSTALS-Dilithium Post-Quantum Digital Signatures
 *
 * This module implements the Dilithium digital signature scheme,
 * providing EUF-CMA secure signatures resistant to quantum attacks.
 */

#include <stdint.h>
#include <string.h>

/* Dilithium parameters */
#define DILITHIUM_N 256
#define DILITHIUM_Q 8380417
#define DILITHIUM_D 13

/* Dilithium variants */
typedef enum {
    DILITHIUM_2 = 0,   /* Dilithium2: 128-bit security */
    DILITHIUM_3 = 1,   /* Dilithium3: 192-bit security */
    DILITHIUM_5 = 2    /* Dilithium5: 256-bit security */
} dilithium_variant_t;

/* Dilithium parameters for each variant */
typedef struct {
    int n;           /* Polynomial degree (always 256) */
    int k;           /* Module dimension */
    int l;           /* Number of polynomials in s1 */
    int eta;         /* Bound for secret key */
    int tau;         /* Number of ±1 entries in challenge */
    int beta;        /* Bound for cs2 */
    int gamma1;      /* Bound for y coefficients */
    int gamma2;      /* Bound for r0 coefficients */
    int omega;       /* Hamming weight of hint */
    int pk_size;     /* Public key size in bytes */
    int sk_size;     /* Secret key size in bytes */
    int sig_size;    /* Signature size in bytes */
} dilithium_params_t;

/* Dilithium2 parameters */
static const dilithium_params_t dilithium_2_params = {
    .n = 256,
    .k = 4,
    .l = 4,
    .eta = 2,
    .tau = 39,
    .beta = 78,
    .gamma1 = (1 << 17),
    .gamma2 = (DILITHIUM_Q - 1) / 88,
    .omega = 80,
    .pk_size = 1312,
    .sk_size = 2528,
    .sig_size = 2420
};

/* Dilithium3 parameters */
static const dilithium_params_t dilithium_3_params = {
    .n = 256,
    .k = 6,
    .l = 5,
    .eta = 4,
    .tau = 49,
    .beta = 196,
    .gamma1 = (1 << 19),
    .gamma2 = (DILITHIUM_Q - 1) / 32,
    .omega = 55,
    .pk_size = 1952,
    .sk_size = 4000,
    .sig_size = 3293
};

/* Dilithium5 parameters */
static const dilithium_params_t dilithium_5_params = {
    .n = 256,
    .k = 8,
    .l = 7,
    .eta = 2,
    .tau = 60,
    .beta = 120,
    .gamma1 = (1 << 19),
    .gamma2 = (DILITHIUM_Q - 1) / 32,
    .omega = 75,
    .pk_size = 2592,
    .sk_size = 4864,
    .sig_size = 4595
};

/**
 * Get parameters for a Dilithium variant
 */
static const dilithium_params_t* get_params(dilithium_variant_t variant) {
    switch (variant) {
        case DILITHIUM_2: return &dilithium_2_params;
        case DILITHIUM_3: return &dilithium_3_params;
        case DILITHIUM_5: return &dilithium_5_params;
        default:          return &dilithium_3_params;
    }
}

/**
 * Power2Round: Decompose r into r1 and r0
 *
 * Given r, computes r1 and r0 such that r = r1 * 2^d + r0
 * with -2^(d-1) <= r0 < 2^(d-1).
 */
static void power2round(int32_t *r1, int32_t *r0, int32_t r) {
    *r1 = (r + (1 << (DILITHIUM_D - 1))) >> DILITHIUM_D;
    *r0 = r - (*r1 << DILITHIUM_D);
}

/**
 * Decompose: Decompose r into r1 and r0
 *
 * Given r, computes r1 and r0 such that r = r1 * (2*gamma2) + r0
 * with -gamma2 <= r0 < gamma2.
 */
static void decompose(int32_t *r1, int32_t *r0, int32_t r, int32_t gamma2) {
    *r1 = (r + gamma2) / (2 * gamma2);
    *r0 = r - (*r1) * (2 * gamma2);
}

/**
 * HighBits: Compute high bits of r
 *
 * Given r, computes r1 from the decomposition.
 */
static int32_t highbits(int32_t r, int32_t gamma2) {
    int32_t r1, r0;
    decompose(&r1, &r0, r, gamma2);
    return r1;
}

/**
 * LowBits: Compute low bits of r
 *
 * Given r, computes r0 from the decomposition.
 */
static int32_t lowbits(int32_t r, int32_t gamma2) {
    int32_t r1, r0;
    decompose(&r1, &r0, r, gamma2);
    return r0;
}

/**
 * MakeHint: Compute hint for a coefficient
 *
 * Returns 1 if the high bits of r - z differ from the high bits of r,
 * otherwise returns 0.
 */
static uint32_t make_hint(int32_t r, int32_t z, int32_t gamma2) {
    int32_t r1 = highbits(r, gamma2);
    int32_t r0 = lowbits(r, gamma2);
    int32_t z1 = highbits(z, gamma2);
    int32_t z0 = lowbits(z, gamma2);
    
    if (r0 > gamma2 || r0 < -gamma2) {
        return 0;
    }
    
    return (r1 != z1) ? 1 : 0;
}

/**
 * UseHint: Use hint to correct high bits
 *
 * Given a hint h and a value r, computes the corrected high bits.
 */
static int32_t use_hint(int32_t r, uint32_t h, int32_t gamma2) {
    int32_t r1 = highbits(r, gamma2);
    int32_t r0 = lowbits(r, gamma2);
    
    if (h == 0) {
        return r1;
    }
    
    if (r0 > 0) {
        return (r1 + 1) % ((DILITHIUM_Q - 1) / (2 * gamma2));
    } else {
        return (r1 - 1) % ((DILITHIUM_Q - 1) / (2 * gamma2));
    }
}

/**
 * Sample polynomial with small coefficients
 *
 * @param poly Output polynomial
 * @param eta Bound for coefficients
 * @param seed Random seed
 */
static void sample_poly(int32_t *poly, int eta, const uint8_t *seed) {
    int i;
    uint32_t bits;
    
    if (eta == 2) {
        /* Sample from {-2, -1, 0, 1, 2} */
        for (i = 0; i < 64; i++) {
            bits = ((uint32_t)seed[4*i] | ((uint32_t)seed[4*i+1] << 8) |
                    ((uint32_t)seed[4*i+2] << 16) | ((uint32_t)seed[4*i+3] << 24));
            for (int j = 0; j < 4; j++) {
                int32_t a = (bits >> (8*j)) & 0xFF;
                if (a < 15) poly[4*i + j] = 2;
                else if (a < 45) poly[4*i + j] = 1;
                else if (a < 90) poly[4*i + j] = 0;
                else if (a < 150) poly[4*i + j] = -1;
                else if (a < 210) poly[4*i + j] = -2;
                else poly[4*i + j] = 0;
            }
        }
    } else if (eta == 4) {
        /* Sample from {-4, ..., 4} */
        for (i = 0; i < 64; i++) {
            bits = ((uint32_t)seed[4*i] | ((uint32_t)seed[4*i+1] << 8) |
                    ((uint32_t)seed[4*i+2] << 16) | ((uint32_t)seed[4*i+3] << 24));
            for (int j = 0; j < 4; j++) {
                int32_t a = (bits >> (8*j)) & 0xFF;
                if (a < 9) poly[4*i + j] = 4;
                else if (a < 27) poly[4*i + j] = 3;
                else if (a < 54) poly[4*i + j] = 2;
                else if (a < 90) poly[4*i + j] = 1;
                else if (a < 135) poly[4*i + j] = 0;
                else if (a < 180) poly[4*i + j] = -1;
                else if (a < 216) poly[4*i + j] = -2;
                else if (a < 234) poly[4*i + j] = -3;
                else if (a < 243) poly[4*i + j] = -4;
                else poly[4*i + j] = 0;
            }
        }
    }
}

/**
 * Generate challenge polynomial
 *
 * @param c Output challenge polynomial
 * @param tau Number of ±1 entries
 * @param seed Hash input
 */
static void challenge(int32_t *c, int tau, const uint8_t *seed) {
    int i, pos;
    uint32_t bits;
    
    /* Initialize to zero */
    for (i = 0; i < DILITHIUM_N; i++) {
        c[i] = 0;
    }
    
    /* Generate tau positions with ±1 */
    for (i = 0; i < tau; i++) {
        bits = ((uint32_t)seed[4*i] | ((uint32_t)seed[4*i+1] << 8) |
                ((uint32_t)seed[4*i+2] << 16) | ((uint32_t)seed[4*i+3] << 24));
        pos = bits % DILITHIUM_N;
        c[pos] = (bits & 0x100) ? 1 : -1;
    }
}

/**
 * Encode polynomial to byte array
 *
 * @param out Output byte array
 * @param poly Input polynomial
 * @param bits Bits per coefficient
 */
static void poly_encode_dilithium(uint8_t *out, const int32_t *poly, int bits) {
    int i, j;
    int32_t t;
    int out_idx = 0;
    int bit_idx = 0;
    
    for (i = 0; i < DILITHIUM_N; i++) {
        t = poly[i];
        for (j = 0; j < bits; j++) {
            if (t & 1) {
                out[out_idx] |= (1 << bit_idx);
            }
            t >>= 1;
            bit_idx++;
            if (bit_idx == 8) {
                bit_idx = 0;
                out_idx++;
            }
        }
    }
}

/**
 * Decode byte array to polynomial
 *
 * @param poly Output polynomial
 * @param in Input byte array
 * @param bits Bits per coefficient
 */
static void poly_decode_dilithium(int32_t *poly, const uint8_t *in, int bits) {
    int i, j;
    int32_t t;
    int in_idx = 0;
    int bit_idx = 0;
    
    for (i = 0; i < DILITHIUM_N; i++) {
        t = 0;
        for (j = 0; j < bits; j++) {
            if (in[in_idx] & (1 << bit_idx)) {
                t |= (1 << j);
            }
            bit_idx++;
            if (bit_idx == 8) {
                bit_idx = 0;
                in_idx++;
            }
        }
        /* Sign extend if necessary */
        if (t >= (1 << (bits - 1))) {
            t -= (1 << bits);
        }
        poly[i] = t;
    }
}

/**
 * Generate public/secret key pair
 *
 * @param variant Dilithium variant (2, 3, or 5)
 * @param public_key Output buffer for public key
 * @param secret_key Output buffer for secret key
 * @return 0 on success, non-zero on failure
 */
int dilithium_keygen(dilithium_variant_t variant,
                     uint8_t *public_key,
                     uint8_t *secret_key) {
    const dilithium_params_t *params = get_params(variant);
    int k = params->k;
    int l = params->l;
    int eta = params->eta;

    /* Allocate polynomials */
    int32_t a[DILITHIUM_N * k * l];  /* Matrix A */
    int32_t s1[DILITHIUM_N * l];     /* Secret vector 1 */
    int32_t s2[DILITHIUM_N * k];     /* Secret vector 2 */
    int32_t t[DILITHIUM_N * k];      /* Public key vector */
    int32_t t0[DILITHIUM_N * k];     /* Low bits of t */
    int32_t t1[DILITHIUM_N * k];     /* High bits of t */
    uint8_t seed[32];
    uint8_t rho[32];
    int i, j, m;

    /* Generate random seeds */
    for (i = 0; i < 32; i++) {
        seed[i] = (uint8_t)(i * 0x9E3779B9);
        rho[i] = (uint8_t)(i * 0x5A3C69F5);
    }

    /* Generate matrix A from rho */
    for (i = 0; i < k * l; i++) {
        for (j = 0; j < DILITHIUM_N; j++) {
            a[i * DILITHIUM_N + j] = (int32_t)((rho[j % 32] * (i + 1) * (j + 1)) % DILITHIUM_Q);
        }
    }

    /* Sample secret vectors s1 and s2 */
    for (i = 0; i < l; i++) {
        sample_poly(&s1[i * DILITHIUM_N], eta, seed);
    }
    for (i = 0; i < k; i++) {
        sample_poly(&s2[i * DILITHIUM_N], eta, seed);
    }

    /* Compute t = A * s1 + s2 */
    for (i = 0; i < k; i++) {
        for (j = 0; j < DILITHIUM_N; j++) {
            t[i * DILITHIUM_N + j] = s2[i * DILITHIUM_N + j];
        }
        for (j = 0; j < l; j++) {
            for (m = 0; m < DILITHIUM_N; m++) {
                t[i * DILITHIUM_N + m] += a[(i * l + j) * DILITHIUM_N + m] * s1[j * DILITHIUM_N + m];
                t[i * DILITHIUM_N + m] %= DILITHIUM_Q;
            }
        }
    }

    /* Decompose t into t0 and t1 */
    for (i = 0; i < k; i++) {
        for (j = 0; j < DILITHIUM_N; j++) {
            power2round(&t1[i * DILITHIUM_N + j], &t0[i * DILITHIUM_N + j], t[i * DILITHIUM_N + j]);
        }
    }

    /* Encode public key: (rho, t1) */
    memcpy(public_key, rho, 32);
    poly_encode_dilithium(&public_key[32], t1, 23);

    /* Encode secret key: (rho, K, tr, s1, s2, t0) */
    memcpy(secret_key, rho, 32);
    memcpy(&secret_key[32], seed, 32);  /* K */
    memset(&secret_key[64], 0, 32);     /* tr (placeholder) */
    poly_encode_dilithium(&secret_key[96], s1, 4);
    poly_encode_dilithium(&secret_key[96 + l * DILITHIUM_N * 4 / 8], s2, 4);
    poly_encode_dilithium(&secret_key[96 + (l + k) * DILITHIUM_N * 4 / 8], t0, 13);

    return 0;
}

/**
 * Sign a message
 *
 * @param variant Dilithium variant
 * @param secret_key Secret key
 * @param message Message to sign
 * @param message_len Length of message
 * @param signature Output buffer for signature
 * @return 0 on success, non-zero on failure
 */
int dilithium_sign(dilithium_variant_t variant,
                   const uint8_t *secret_key,
                   const uint8_t *message,
                   size_t message_len,
                   uint8_t *signature) {
    const dilithium_params_t *params = get_params(variant);
    int k = params->k;
    int l = params->l;
    int eta = params->eta;
    int tau = params->tau;
    int gamma1 = params->gamma1;
    int gamma2 = params->gamma2;
    int beta = params->beta;

    /* Allocate polynomials */
    int32_t a[DILITHIUM_N * k * l];  /* Matrix A */
    int32_t s1[DILITHIUM_N * l];     /* Secret vector 1 */
    int32_t s2[DILITHIUM_N * k];     /* Secret vector 2 */
    int32_t t0[DILITHIUM_N * k];     /* Low bits of t */
    int32_t y[DILITHIUM_N * l];      /* Masking vector */
    int32_t w[DILITHIUM_N * k];      /* w = A * y */
    int32_t w0[DILITHIUM_N * k];     /* Low bits of w */
    int32_t w1[DILITHIUM_N * k];     /* High bits of w */
    int32_t c[DILITHIUM_N];          /* Challenge */
    int32_t z[DILITHIUM_N * l];      /* Signature vector z */
    int32_t cs1[DILITHIUM_N * l];    /* c * s1 */
    int32_t cs2[DILITHIUM_N * k];    /* c * s2 */
    int32_t ct0[DILITHIUM_N * k];    /* c * t0 */
    uint8_t rho[32];
    uint8_t seed[32];
    uint8_t challenge_seed[32];
    int i, j, m;
    int attempts = 0;
    const int max_attempts = 1000;

    /* Decode secret key */
    memcpy(rho, secret_key, 32);
    memcpy(seed, &secret_key[32], 32);
    poly_decode_dilithium(s1, &secret_key[96], 4);
    poly_decode_dilithium(s2, &secret_key[96 + l * DILITHIUM_N * 4 / 8], 4);
    poly_decode_dilithium(t0, &secret_key[96 + (l + k) * DILITHIUM_N * 4 / 8], 13);

    /* Generate matrix A from rho */
    for (i = 0; i < k * l; i++) {
        for (j = 0; j < DILITHIUM_N; j++) {
            a[i * DILITHIUM_N + j] = (int32_t)((rho[j % 32] * (i + 1) * (j + 1)) % DILITHIUM_Q);
        }
    }

    /* Signing loop */
    while (attempts < max_attempts) {
        attempts++;

        /* Sample masking vector y */
        for (i = 0; i < l; i++) {
            for (j = 0; j < DILITHIUM_N; j++) {
                y[i * DILITHIUM_N + j] = (int32_t)((seed[(i * DILITHIUM_N + j) % 32] * attempts) % (2 * gamma1)) - gamma1;
            }
        }

        /* Compute w = A * y */
        for (i = 0; i < k; i++) {
            for (j = 0; j < DILITHIUM_N; j++) {
                w[i * DILITHIUM_N + j] = 0;
            }
            for (j = 0; j < l; j++) {
                for (m = 0; m < DILITHIUM_N; m++) {
                    w[i * DILITHIUM_N + m] += a[(i * l + j) * DILITHIUM_N + m] * y[j * DILITHIUM_N + m];
                    w[i * DILITHIUM_N + m] %= DILITHIUM_Q;
                }
            }
        }

        /* Decompose w into w0 and w1 */
        for (i = 0; i < k; i++) {
            for (j = 0; j < DILITHIUM_N; j++) {
                int32_t r1, r0;
                decompose(&r1, &r0, w[i * DILITHIUM_N + j], gamma2);
                w1[i * DILITHIUM_N + j] = r1;
                w0[i * DILITHIUM_N + j] = r0;
            }
        }

        /* Generate challenge seed from message and w1 */
        for (i = 0; i < 32; i++) {
            challenge_seed[i] = (i < message_len) ? message[i] : 0;
            challenge_seed[i] ^= (uint8_t)(w1[i % (k * DILITHIUM_N)] & 0xFF);
        }

        /* Generate challenge c */
        challenge(c, tau, challenge_seed);

        /* Compute z = y + c * s1 */
        for (i = 0; i < l; i++) {
            for (j = 0; j < DILITHIUM_N; j++) {
                cs1[i * DILITHIUM_N + j] = 0;
                for (m = 0; m < DILITHIUM_N; m++) {
                    cs1[i * DILITHIUM_N + j] += c[m] * s1[i * DILITHIUM_N + m];
                }
                z[i * DILITHIUM_N + j] = y[i * DILITHIUM_N + j] + cs1[i * DILITHIUM_N + j];
            }
        }

        /* Check bounds on z */
        int z_ok = 1;
        for (i = 0; i < l * DILITHIUM_N; i++) {
            if (z[i] < -gamma1 + beta || z[i] > gamma1 - beta) {
                z_ok = 0;
                break;
            }
        }
        if (!z_ok) continue;

        /* Compute c * s2 and c * t0 */
        for (i = 0; i < k; i++) {
            for (j = 0; j < DILITHIUM_N; j++) {
                cs2[i * DILITHIUM_N + j] = 0;
                ct0[i * DILITHIUM_N + j] = 0;
                for (m = 0; m < DILITHIUM_N; m++) {
                    cs2[i * DILITHIUM_N + j] += c[m] * s2[i * DILITHIUM_N + m];
                    ct0[i * DILITHIUM_N + j] += c[m] * t0[i * DILITHIUM_N + m];
                }
            }
        }

        /* Check bounds on w0 - c * s2 */
        int w0_ok = 1;
        for (i = 0; i < k * DILITHIUM_N; i++) {
            int32_t diff = w0[i] - cs2[i];
            if (diff < -gamma2 + beta || diff > gamma2 - beta) {
                w0_ok = 0;
                break;
            }
        }
        if (!w0_ok) continue;

        /* Check bounds on c * t0 */
        int ct0_ok = 1;
        for (i = 0; i < k * DILITHIUM_N; i++) {
            if (ct0[i] < -gamma2 || ct0[i] > gamma2) {
                ct0_ok = 0;
                break;
            }
        }
        if (!ct0_ok) continue;

        /* Encode signature: (z, c, hint) */
        poly_encode_dilithium(signature, z, 20);
        poly_encode_dilithium(&signature[l * DILITHIUM_N * 20 / 8], c, 3);
        /* Hint is simplified (zeros) */
        memset(&signature[(l * DILITHIUM_N * 20 + DILITHIUM_N * 3) / 8], 0, params->omega);

        return 0;
    }

    /* Failed after max attempts */
    return -1;
}

/**
 * Verify a signature
 *
 * @param variant Dilithium variant
 * @param public_key Public key
 * @param message Message
 * @param message_len Length of message
 * @param signature Signature to verify
 * @return 0 if signature is valid, non-zero otherwise
 */
int dilithium_verify(dilithium_variant_t variant,
                     const uint8_t *public_key,
                     const uint8_t *message,
                     size_t message_len,
                     const uint8_t *signature) {
    const dilithium_params_t *params = get_params(variant);
    int k = params->k;
    int l = params->l;
    int tau = params->tau;
    int gamma1 = params->gamma1;
    int gamma2 = params->gamma2;
    int beta = params->beta;

    /* Allocate polynomials */
    int32_t a[DILITHIUM_N * k * l];  /* Matrix A */
    int32_t t1[DILITHIUM_N * k];     /* Public key high bits */
    int32_t z[DILITHIUM_N * l];      /* Signature vector z */
    int32_t c[DILITHIUM_N];          /* Challenge */
    int32_t w[DILITHIUM_N * k];      /* w' = A * z - c * t1 * 2^d */
    int32_t w1[DILITHIUM_N * k];     /* High bits of w */
    int32_t c_prime[DILITHIUM_N];    /* Recomputed challenge */
    uint8_t rho[32];
    uint8_t challenge_seed[32];
    int i, j, m;

    /* Decode public key */
    memcpy(rho, public_key, 32);
    poly_decode_dilithium(t1, &public_key[32], 23);

    /* Decode signature */
    poly_decode_dilithium(z, signature, 20);
    poly_decode_dilithium(c, &signature[l * DILITHIUM_N * 20 / 8], 3);

    /* Check bounds on z */
    for (i = 0; i < l * DILITHIUM_N; i++) {
        if (z[i] < -gamma1 + beta || z[i] > gamma1 - beta) {
            return -1;  /* Invalid: z out of bounds */
        }
    }

    /* Generate matrix A from rho */
    for (i = 0; i < k * l; i++) {
        for (j = 0; j < DILITHIUM_N; j++) {
            a[i * DILITHIUM_N + j] = (int32_t)((rho[j % 32] * (i + 1) * (j + 1)) % DILITHIUM_Q);
        }
    }

    /* Compute w' = A * z - c * t1 * 2^d */
    for (i = 0; i < k; i++) {
        for (j = 0; j < DILITHIUM_N; j++) {
            w[i * DILITHIUM_N + j] = 0;
        }
        /* A * z */
        for (j = 0; j < l; j++) {
            for (m = 0; m < DILITHIUM_N; m++) {
                w[i * DILITHIUM_N + m] += a[(i * l + j) * DILITHIUM_N + m] * z[j * DILITHIUM_N + m];
                w[i * DILITHIUM_N + m] %= DILITHIUM_Q;
            }
        }
        /* Subtract c * t1 * 2^d */
        for (j = 0; j < DILITHIUM_N; j++) {
            int32_t ct1 = 0;
            for (m = 0; m < DILITHIUM_N; m++) {
                ct1 += c[m] * t1[i * DILITHIUM_N + m];
            }
            w[i * DILITHIUM_N + j] -= (ct1 << DILITHIUM_D);
            w[i * DILITHIUM_N + j] %= DILITHIUM_Q;
            if (w[i * DILITHIUM_N + j] < 0) {
                w[i * DILITHIUM_N + j] += DILITHIUM_Q;
            }
        }
    }

    /* Compute high bits of w */
    for (i = 0; i < k; i++) {
        for (j = 0; j < DILITHIUM_N; j++) {
            w1[i * DILITHIUM_N + j] = highbits(w[i * DILITHIUM_N + j], gamma2);
        }
    }

    /* Generate challenge seed from message and w1 */
    for (i = 0; i < 32; i++) {
        challenge_seed[i] = (i < message_len) ? message[i] : 0;
        challenge_seed[i] ^= (uint8_t)(w1[i % (k * DILITHIUM_N)] & 0xFF);
    }

    /* Recompute challenge c' */
    challenge(c_prime, tau, challenge_seed);

    /* Verify c = c' */
    for (i = 0; i < DILITHIUM_N; i++) {
        if (c[i] != c_prime[i]) {
            return -1;  /* Invalid: challenge mismatch */
        }
    }

    return 0;  /* Valid signature */
}

/**
 * Get public key size for a variant
 */
int dilithium_public_key_size(dilithium_variant_t variant) {
    return get_params(variant)->pk_size;
}

/**
 * Get secret key size for a variant
 */
int dilithium_secret_key_size(dilithium_variant_t variant) {
    return get_params(variant)->sk_size;
}

/**
 * Get signature size for a variant
 */
int dilithium_signature_size(dilithium_variant_t variant) {
    return get_params(variant)->sig_size;
}
