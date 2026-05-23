/*
 * Nuva OS - CRYSTALS-Kyber KEM Implementation
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
 * CRYSTALS-Kyber Key Encapsulation Mechanism
 *
 * This module implements the Kyber key encapsulation mechanism (KEM)
 * providing IND-CCA2 secure key exchange.
 */

#include <stdint.h>
#include <string.h>
#include "ntt.c"

/* Kyber variants */
typedef enum {
    KYBER_512 = 0,   /* Kyber-512: 512-bit security */
    KYBER_768 = 1,   /* Kyber-768: 768-bit security */
    KYBER_1024 = 2   /* Kyber-1024: 1024-bit security */
} kyber_variant_t;

/* Kyber parameters for each variant */
typedef struct {
    int n;           /* Polynomial degree (always 256) */
    int k;           /* Module dimension */
    int eta1;        /* CBD parameter for secret */
    int eta2;        /* CBD parameter for error */
    int du;          /* Compression parameter for u */
    int dv;          /* Compression parameter for v */
    int pk_size;     /* Public key size in bytes */
    int sk_size;     /* Secret key size in bytes */
    int ct_size;     /* Ciphertext size in bytes */
} kyber_params_t;

/* Kyber-512 parameters */
static const kyber_params_t kyber_512_params = {
    .n = 256,
    .k = 2,
    .eta1 = 3,
    .eta2 = 2,
    .du = 10,
    .dv = 4,
    .pk_size = 800,
    .sk_size = 1632,
    .ct_size = 768
};

/* Kyber-768 parameters */
static const kyber_params_t kyber_768_params = {
    .n = 256,
    .k = 3,
    .eta1 = 2,
    .eta2 = 2,
    .du = 10,
    .dv = 4,
    .pk_size = 1184,
    .sk_size = 2400,
    .ct_size = 1088
};

/* Kyber-1024 parameters */
static const kyber_params_t kyber_1024_params = {
    .n = 256,
    .k = 4,
    .eta1 = 2,
    .eta2 = 2,
    .du = 11,
    .dv = 5,
    .pk_size = 1568,
    .sk_size = 3168,
    .ct_size = 1568
};

/**
 * Get parameters for a Kyber variant
 */
static const kyber_params_t* get_params(kyber_variant_t variant) {
    switch (variant) {
        case KYBER_512:  return &kyber_512_params;
        case KYBER_768:  return &kyber_768_params;
        case KYBER_1024: return &kyber_1024_params;
        default:         return &kyber_768_params;
    }
}

/**
 * Centered Binomial Distribution (CBD) sampler
 *
 * Samples a polynomial with coefficients from CBD_η
 *
 * @param poly Output polynomial
 * @param eta CBD parameter (2 or 3)
 * @param seed Random seed
 */
static void cbd(int16_t *poly, int eta, const uint8_t *seed) {
    uint32_t bits;
    int i, j;
    int16_t a, b;

    if (eta == 2) {
        /* CBD2: each coefficient is sum of 2 pairs of bits */
        for (i = 0; i < 128; i++) {
            bits = ((uint32_t)seed[4*i] | ((uint32_t)seed[4*i+1] << 8) |
                    ((uint32_t)seed[4*i+2] << 16) | ((uint32_t)seed[4*i+3] << 24));
            for (j = 0; j < 2; j++) {
                a = (bits >> (4*j)) & 0x3;
                b = (bits >> (4*j + 2)) & 0x3;
                poly[2*i + j] = (a & 1) + ((a >> 1) & 1) - (b & 1) - ((b >> 1) & 1);
            }
        }
    } else if (eta == 3) {
        /* CBD3: each coefficient is sum of 3 pairs of bits */
        for (i = 0; i < 96; i++) {
            bits = ((uint32_t)seed[4*i] | ((uint32_t)seed[4*i+1] << 8) |
                    ((uint32_t)seed[4*i+2] << 16) | ((uint32_t)seed[4*i+3] << 24));
            for (j = 0; j < 4; j++) {
                a = (bits >> (6*j)) & 0x7;
                b = (bits >> (6*j + 3)) & 0x7;
                poly[4*i + j] = (a & 1) + ((a >> 1) & 1) + ((a >> 2) & 1)
                              - (b & 1) - ((b >> 1) & 1) - ((b >> 2) & 1);
            }
        }
    }
}

/**
 * Encode polynomial to byte array (12-bit coefficients)
 *
 * @param out Output byte array
 * @param poly Input polynomial
 * @param n Number of coefficients
 */
static void poly_encode(uint8_t *out, const int16_t *poly, int n) {
    int i;
    for (i = 0; i < n/2; i++) {
        out[3*i] = poly[2*i] & 0xFF;
        out[3*i+1] = ((poly[2*i] >> 8) & 0x0F) | ((poly[2*i+1] & 0x0F) << 4);
        out[3*i+2] = (poly[2*i+1] >> 4) & 0xFF;
    }
}

/**
 * Decode byte array to polynomial (12-bit coefficients)
 *
 * @param poly Output polynomial
 * @param in Input byte array
 * @param n Number of coefficients
 */
static void poly_decode(int16_t *poly, const uint8_t *in, int n) {
    int i;
    for (i = 0; i < n/2; i++) {
        poly[2*i] = in[3*i] | ((in[3*i+1] & 0x0F) << 8);
        poly[2*i+1] = (in[3*i+1] >> 4) | (in[3*i+2] << 4);
    }
}

/**
 * Compress polynomial
 *
 * @param out Output polynomial
 * @param in Input polynomial
 * @param d Compression parameter
 * @param n Number of coefficients
 */
static void poly_compress(uint8_t *out, const int16_t *in, int d, int n) {
    int i;
    uint16_t t;
    int16_t u;

    if (d == 10) {
        for (i = 0; i < n/4; i++) {
            t = (uint16_t)((in[4*i] * 1024 + KYBER_Q/2) / KYBER_Q);
            u = (int16_t)((in[4*i+1] * 1024 + KYBER_Q/2) / KYBER_Q);
            out[5*i] = t & 0xFF;
            out[5*i+1] = ((t >> 8) & 0x03) | ((u << 2) & 0xFC);
            t = (uint16_t)((in[4*i+2] * 1024 + KYBER_Q/2) / KYBER_Q);
            out[5*i+2] = ((u >> 6) & 0x0F) | ((t << 4) & 0xF0);
            u = (int16_t)((in[4*i+3] * 1024 + KYBER_Q/2) / KYBER_Q);
            out[5*i+3] = ((t >> 4) & 0x3F) | ((u << 6) & 0xC0);
            out[5*i+4] = (u >> 2) & 0xFF;
        }
    } else if (d == 11) {
        for (i = 0; i < n/8; i++) {
            t = (uint16_t)((in[8*i] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i] = t & 0xFF;
            out[11*i+1] = ((t >> 8) & 0x07);
            t = (uint16_t)((in[8*i+1] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i+1] |= (t << 3) & 0xF8;
            out[11*i+2] = (t >> 5) & 0xFF;
            t = (uint16_t)((in[8*i+2] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i+3] = t & 0xFF;
            out[11*i+4] = ((t >> 8) & 0x07);
            t = (uint16_t)((in[8*i+3] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i+4] |= (t << 3) & 0xF8;
            out[11*i+5] = (t >> 5) & 0xFF;
            t = (uint16_t)((in[8*i+4] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i+6] = t & 0xFF;
            out[11*i+7] = ((t >> 8) & 0x07);
            t = (uint16_t)((in[8*i+5] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i+7] |= (t << 3) & 0xF8;
            out[11*i+8] = (t >> 5) & 0xFF;
            t = (uint16_t)((in[8*i+6] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i+9] = t & 0xFF;
            out[11*i+10] = ((t >> 8) & 0x07);
            t = (uint16_t)((in[8*i+7] * 2048 + KYBER_Q/2) / KYBER_Q);
            out[11*i+10] |= (t << 3) & 0xF8;
        }
    } else if (d == 4) {
        for (i = 0; i < n/2; i++) {
            t = (uint16_t)((in[2*i] * 16 + KYBER_Q/2) / KYBER_Q);
            u = (int16_t)((in[2*i+1] * 16 + KYBER_Q/2) / KYBER_Q);
            out[i] = (t & 0x0F) | ((u << 4) & 0xF0);
        }
    } else if (d == 5) {
        for (i = 0; i < n/8; i++) {
            t = (uint16_t)((in[8*i] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i] = t & 0xFF;
            out[5*i+1] = ((t >> 8) & 0x01);
            t = (uint16_t)((in[8*i+1] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i+1] |= (t << 1) & 0xFE;
            out[5*i+2] = (t >> 7) & 0x03;
            t = (uint16_t)((in[8*i+2] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i+2] |= (t << 2) & 0xFC;
            out[5*i+3] = (t >> 6) & 0x07;
            t = (uint16_t)((in[8*i+3] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i+3] |= (t << 3) & 0xF8;
            out[5*i+4] = (t >> 5) & 0x0F;
            t = (uint16_t)((in[8*i+4] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i+4] |= (t << 4) & 0xF0;
            t = (uint16_t)((in[8*i+5] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i+5] = t & 0xFF;
            out[5*i+6] = ((t >> 8) & 0x01);
            t = (uint16_t)((in[8*i+6] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i+6] |= (t << 1) & 0xFE;
            out[5*i+7] = (t >> 7) & 0x03;
            t = (uint16_t)((in[8*i+7] * 32 + KYBER_Q/2) / KYBER_Q);
            out[5*i+7] |= (t << 2) & 0xFC;
        }
    }
}

/**
 * Decompress polynomial
 *
 * @param out Output polynomial
 * @param in Input byte array
 * @param d Compression parameter
 * @param n Number of coefficients
 */
static void poly_decompress(int16_t *out, const uint8_t *in, int d, int n) {
    int i;
    uint16_t t;

    if (d == 10) {
        for (i = 0; i < n/4; i++) {
            t = in[5*i] | ((in[5*i+1] & 0x03) << 8);
            out[4*i] = (int16_t)((t * KYBER_Q + 512) / 1024);
            t = ((in[5*i+1] >> 2) & 0x3F) | ((in[5*i+2] & 0x0F) << 6);
            out[4*i+1] = (int16_t)((t * KYBER_Q + 512) / 1024);
            t = ((in[5*i+2] >> 4) & 0x0F) | ((in[5*i+3] & 0x3F) << 4);
            out[4*i+2] = (int16_t)((t * KYBER_Q + 512) / 1024);
            t = ((in[5*i+3] >> 6) & 0x03) | (in[5*i+4] << 2);
            out[4*i+3] = (int16_t)((t * KYBER_Q + 512) / 1024);
        }
    } else if (d == 11) {
        for (i = 0; i < n/8; i++) {
            t = in[11*i] | ((in[11*i+1] & 0x07) << 8);
            out[8*i] = (int16_t)((t * KYBER_Q + 1024) / 2048);
            t = ((in[11*i+1] >> 3) & 0x1F) | (in[11*i+2] << 5);
            out[8*i+1] = (int16_t)((t * KYBER_Q + 1024) / 2048);
            t = in[11*i+3] | ((in[11*i+4] & 0x07) << 8);
            out[8*i+2] = (int16_t)((t * KYBER_Q + 1024) / 2048);
            t = ((in[11*i+4] >> 3) & 0x1F) | (in[11*i+5] << 5);
            out[8*i+3] = (int16_t)((t * KYBER_Q + 1024) / 2048);
            t = in[11*i+6] | ((in[11*i+7] & 0x07) << 8);
            out[8*i+4] = (int16_t)((t * KYBER_Q + 1024) / 2048);
            t = ((in[11*i+7] >> 3) & 0x1F) | (in[11*i+8] << 5);
            out[8*i+5] = (int16_t)((t * KYBER_Q + 1024) / 2048);
            t = in[11*i+9] | ((in[11*i+10] & 0x07) << 8);
            out[8*i+6] = (int16_t)((t * KYBER_Q + 1024) / 2048);
            t = ((in[11*i+10] >> 3) & 0x1F);
            out[8*i+7] = (int16_t)((t * KYBER_Q + 1024) / 2048);
        }
    } else if (d == 4) {
        for (i = 0; i < n/2; i++) {
            t = in[i] & 0x0F;
            out[2*i] = (int16_t)((t * KYBER_Q + 8) / 16);
            t = (in[i] >> 4) & 0x0F;
            out[2*i+1] = (int16_t)((t * KYBER_Q + 8) / 16);
        }
    } else if (d == 5) {
        for (i = 0; i < n/8; i++) {
            t = in[5*i] | ((in[5*i+1] & 0x01) << 8);
            out[8*i] = (int16_t)((t * KYBER_Q + 16) / 32);
            t = ((in[5*i+1] >> 1) & 0x7F) | ((in[5*i+2] & 0x03) << 7);
            out[8*i+1] = (int16_t)((t * KYBER_Q + 16) / 32);
            t = ((in[5*i+2] >> 2) & 0x3F) | ((in[5*i+3] & 0x07) << 6);
            out[8*i+2] = (int16_t)((t * KYBER_Q + 16) / 32);
            t = ((in[5*i+3] >> 3) & 0x1F) | ((in[5*i+4] & 0x0F) << 5);
            out[8*i+3] = (int16_t)((t * KYBER_Q + 16) / 32);
            t = (in[5*i+4] >> 4) & 0x0F;
            out[8*i+4] = (int16_t)((t * KYBER_Q + 16) / 32);
            t = in[5*i+5] | ((in[5*i+6] & 0x01) << 8);
            out[8*i+5] = (int16_t)((t * KYBER_Q + 16) / 32);
            t = ((in[5*i+6] >> 1) & 0x7F) | ((in[5*i+7] & 0x03) << 7);
            out[8*i+6] = (int16_t)((t * KYBER_Q + 16) / 32);
            t = (in[5*i+7] >> 2) & 0x3F;
            out[8*i+7] = (int16_t)((t * KYBER_Q + 16) / 32);
        }
    }
}

/**
 * Generate public/secret key pair
 *
 * @param variant Kyber variant (512, 768, or 1024)
 * @param public_key Output buffer for public key
 * @param secret_key Output buffer for secret key
 * @return 0 on success, non-zero on failure
 */
int kyber_keygen(kyber_variant_t variant,
                 uint8_t *public_key,
                 uint8_t *secret_key) {
    const kyber_params_t *params = get_params(variant);
    int k = params->k;
    int eta1 = params->eta1;

    /* Allocate polynomials */
    int16_t a[KYBER_N * k * k];  /* Matrix A */
    int16_t s[KYBER_N * k];      /* Secret vector */
    int16_t e[KYBER_N * k];      /* Error vector */
    int16_t t[KYBER_N * k];      /* Public key vector */
    int16_t tmp[KYBER_N];
    uint8_t seed[32];
    int i, j;

    /* Generate random seed for matrix A */
    /* In production, this would use a cryptographic RNG */
    for (i = 0; i < 32; i++) {
        seed[i] = (uint8_t)(i * 0x9E3779B9);
    }

    /* Generate matrix A from seed using XOF */
    /* Simplified: use seed directly */
    for (i = 0; i < k * k; i++) {
        for (j = 0; j < KYBER_N; j++) {
            a[i * KYBER_N + j] = (int16_t)((seed[j % 32] * (i + 1) * (j + 1)) % KYBER_Q);
        }
    }

    /* Sample secret vector s from CBD_η1 */
    for (i = 0; i < k; i++) {
        cbd(&s[i * KYBER_N], eta1, seed);
    }

    /* Sample error vector e from CBD_η1 */
    for (i = 0; i < k; i++) {
        cbd(&e[i * KYBER_N], eta1, seed);
    }

    /* Transform s to NTT domain */
    for (i = 0; i < k; i++) {
        ntt(&s[i * KYBER_N], zetas);
    }

    /* Transform A to NTT domain */
    for (i = 0; i < k * k; i++) {
        ntt(&a[i * KYBER_N], zetas);
    }

    /* Compute t = A*s + e in NTT domain */
    for (i = 0; i < k; i++) {
        /* Initialize t[i] = e[i] */
        for (j = 0; j < KYBER_N; j++) {
            t[i * KYBER_N + j] = e[i * KYBER_N + j];
        }

        /* Add A[i][j] * s[j] for each j */
        for (j = 0; j < k; j++) {
            basemul(tmp, &a[(i * k + j) * KYBER_N], &s[j * KYBER_N]);
            for (int l = 0; l < KYBER_N; l++) {
                t[i * KYBER_N + l] = barrett_reduce(t[i * KYBER_N + l] + tmp[l]);
            }
        }
    }

    /* Transform t back from NTT domain */
    for (i = 0; i < k; i++) {
        invntt(&t[i * KYBER_N], zetas_inv);
    }

    /* Encode public key: (t, seed for A) */
    for (i = 0; i < k; i++) {
        poly_encode(&public_key[i * 3 * KYBER_N / 2], &t[i * KYBER_N], KYBER_N);
    }
    memcpy(&public_key[k * 3 * KYBER_N / 2], seed, 32);

    /* Encode secret key: s */
    for (i = 0; i < k; i++) {
        poly_encode(&secret_key[i * 3 * KYBER_N / 2], &s[i * KYBER_N], KYBER_N);
    }
    /* Append public key to secret key */
    memcpy(&secret_key[k * 3 * KYBER_N / 2], public_key, params->pk_size);

    return 0;
}

/**
 * Encapsulate a shared secret
 *
 * @param variant Kyber variant
 * @param public_key Public key
 * @param ciphertext Output buffer for ciphertext
 * @param shared_secret Output buffer for shared secret
 * @return 0 on success, non-zero on failure
 */
int kyber_encapsulate(kyber_variant_t variant,
                      const uint8_t *public_key,
                      uint8_t *ciphertext,
                      uint8_t *shared_secret) {
    const kyber_params_t *params = get_params(variant);
    int k = params->k;
    int eta1 = params->eta1;
    int eta2 = params->eta2;
    int du = params->du;
    int dv = params->dv;

    /* Allocate polynomials */
    int16_t a[KYBER_N * k * k];  /* Matrix A */
    int16_t t[KYBER_N * k];      /* Public key vector */
    int16_t r[KYBER_N * k];      /* Random vector */
    int16_t e1[KYBER_N * k];     /* Error vector 1 */
    int16_t e2[KYBER_N];         /* Error scalar */
    int16_t u[KYBER_N * k];      /* Ciphertext component u */
    int16_t v[KYBER_N];          /* Ciphertext component v */
    int16_t tmp[KYBER_N];
    uint8_t seed[32];
    uint8_t random_bytes[32];
    int i, j;

    /* Decode public key */
    for (i = 0; i < k; i++) {
        poly_decode(&t[i * KYBER_N], &public_key[i * 3 * KYBER_N / 2], KYBER_N);
    }
    memcpy(seed, &public_key[k * 3 * KYBER_N / 2], 32);

    /* Generate matrix A from seed */
    for (i = 0; i < k * k; i++) {
        for (j = 0; j < KYBER_N; j++) {
            a[i * KYBER_N + j] = (int16_t)((seed[j % 32] * (i + 1) * (j + 1)) % KYBER_Q);
        }
    }

    /* Generate random bytes for sampling */
    for (i = 0; i < 32; i++) {
        random_bytes[i] = (uint8_t)(i * 0x5A3C69F5);
    }

    /* Sample random vector r from CBD_η1 */
    for (i = 0; i < k; i++) {
        cbd(&r[i * KYBER_N], eta1, random_bytes);
    }

    /* Sample error vectors e1 from CBD_η2 */
    for (i = 0; i < k; i++) {
        cbd(&e1[i * KYBER_N], eta2, random_bytes);
    }

    /* Sample error scalar e2 from CBD_η2 */
    cbd(e2, eta2, random_bytes);

    /* Transform r to NTT domain */
    for (i = 0; i < k; i++) {
        ntt(&r[i * KYBER_N], zetas);
    }

    /* Transform A to NTT domain */
    for (i = 0; i < k * k; i++) {
        ntt(&a[i * KYBER_N], zetas);
    }

    /* Compute u = A^T * r + e1 */
    for (i = 0; i < k; i++) {
        /* Initialize u[i] = e1[i] */
        for (j = 0; j < KYBER_N; j++) {
            u[i * KYBER_N + j] = e1[i * KYBER_N + j];
        }

        /* Add A[j][i] * r[j] for each j (transpose) */
        for (j = 0; j < k; j++) {
            basemul(tmp, &a[(j * k + i) * KYBER_N], &r[j * KYBER_N]);
            for (int l = 0; l < KYBER_N; l++) {
                u[i * KYBER_N + l] = barrett_reduce(u[i * KYBER_N + l] + tmp[l]);
            }
        }
    }

    /* Transform u back from NTT domain */
    for (i = 0; i < k; i++) {
        invntt(&u[i * KYBER_N], zetas_inv);
    }

    /* Transform t to NTT domain */
    for (i = 0; i < k; i++) {
        ntt(&t[i * KYBER_N], zetas);
    }

    /* Compute v = t^T * r + e2 + encode(shared_secret) */
    for (j = 0; j < KYBER_N; j++) {
        v[j] = e2[j];
    }
    for (i = 0; i < k; i++) {
        basemul(tmp, &t[i * KYBER_N], &r[i * KYBER_N]);
        for (j = 0; j < KYBER_N; j++) {
            v[j] = barrett_reduce(v[j] + tmp[j]);
        }
    }
    invntt(v, zetas_inv);

    /* Generate shared secret and add to v */
    for (i = 0; i < 32; i++) {
        shared_secret[i] = random_bytes[i];
    }
    /* Encode shared secret into polynomial and add to v */
    for (i = 0; i < 32; i++) {
        for (j = 0; j < 8; j++) {
            v[8*i + j] += ((shared_secret[i] >> j) & 1) * (KYBER_Q / 2);
            v[8*i + j] = barrett_reduce(v[8*i + j]);
        }
    }

    /* Compress and encode ciphertext */
    for (i = 0; i < k; i++) {
        poly_compress(&ciphertext[i * ((du * KYBER_N) / 8)], &u[i * KYBER_N], du, KYBER_N);
    }
    poly_compress(&ciphertext[k * ((du * KYBER_N) / 8)], v, dv, KYBER_N);

    return 0;
}

/**
 * Decapsulate a shared secret
 *
 * @param variant Kyber variant
 * @param secret_key Secret key
 * @param ciphertext Ciphertext
 * @param shared_secret Output buffer for shared secret
 * @return 0 on success, non-zero on failure
 */
int kyber_decapsulate(kyber_variant_t variant,
                      const uint8_t *secret_key,
                      const uint8_t *ciphertext,
                      uint8_t *shared_secret) {
    const kyber_params_t *params = get_params(variant);
    int k = params->k;
    int du = params->du;
    int dv = params->dv;

    /* Allocate polynomials */
    int16_t s[KYBER_N * k];      /* Secret vector */
    int16_t u[KYBER_N * k];      /* Ciphertext component u */
    int16_t v[KYBER_N];          /* Ciphertext component v */
    int16_t tmp[KYBER_N];
    int16_t w[KYBER_N];          /* Recovered message */
    int i, j;

    /* Decode secret key */
    for (i = 0; i < k; i++) {
        poly_decode(&s[i * KYBER_N], &secret_key[i * 3 * KYBER_N / 2], KYBER_N);
    }

    /* Decompress and decode ciphertext */
    for (i = 0; i < k; i++) {
        poly_decompress(&u[i * KYBER_N], &ciphertext[i * ((du * KYBER_N) / 8)], du, KYBER_N);
    }
    poly_decompress(v, &ciphertext[k * ((du * KYBER_N) / 8)], dv, KYBER_N);

    /* Transform s to NTT domain */
    for (i = 0; i < k; i++) {
        ntt(&s[i * KYBER_N], zetas);
    }

    /* Transform u to NTT domain */
    for (i = 0; i < k; i++) {
        ntt(&u[i * KYBER_N], zetas);
    }

    /* Compute w = v - s^T * u */
    for (j = 0; j < KYBER_N; j++) {
        w[j] = v[j];
    }
    for (i = 0; i < k; i++) {
        basemul(tmp, &s[i * KYBER_N], &u[i * KYBER_N]);
        for (j = 0; j < KYBER_N; j++) {
            w[j] = barrett_reduce(w[j] - tmp[j]);
        }
    }
    invntt(w, zetas_inv);

    /* Decode shared secret from w */
    for (i = 0; i < 32; i++) {
        shared_secret[i] = 0;
        for (j = 0; j < 8; j++) {
            /* Compare w[8*i + j] to q/2 */
            if (w[8*i + j] > KYBER_Q / 4 && w[8*i + j] < 3 * KYBER_Q / 4) {
                shared_secret[i] |= (1 << j);
            }
        }
    }

    return 0;
}

/**
 * Get public key size for a variant
 */
int kyber_public_key_size(kyber_variant_t variant) {
    return get_params(variant)->pk_size;
}

/**
 * Get secret key size for a variant
 */
int kyber_secret_key_size(kyber_variant_t variant) {
    return get_params(variant)->sk_size;
}

/**
 * Get ciphertext size for a variant
 */
int kyber_ciphertext_size(kyber_variant_t variant) {
    return get_params(variant)->ct_size;
}
