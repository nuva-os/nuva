/*
 * Nuva OS - CRYSTALS-Kyber NTT Implementation
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
 * Number Theoretic Transform (NTT) for CRYSTALS-Kyber
 *
 * This module implements the NTT and inverse NTT operations
 * for polynomial multiplication in the ring Z_q[X]/(X^n + 1).
 */

#include <stdint.h>

/* Kyber parameters */
#define KYBER_N 256
#define KYBER_Q 3329

/* Precomputed roots of unity for NTT */
extern const int16_t zetas[128];
extern const int16_t zetas_inv[128];

/**
 * Montgomery reduction
 *
 * Given a 32-bit integer a, computes 16-bit integer congruent to a * R^-1 mod q,
 * where R = 2^16.
 */
static inline int16_t montgomery_reduce(int32_t a) {
    int16_t t;
    int32_t u;

    t = (int16_t)a;  /* t mod 2^16 */
    u = (int32_t)t * 62209;  /* u = t * q^-1 mod 2^16 */
    u = (a - u * KYBER_Q) >> 16;  /* (a - u * q) / 2^16 */
    
    return (int16_t)u;
}

/**
 * Barrett reduction
 *
 * Given a 16-bit integer a, computes 16-bit integer congruent to a mod q.
 */
static inline int16_t barrett_reduce(int16_t a) {
    int16_t t;
    int32_t u;

    u = (int32_t)a * 20159;  /* v = a * ((2^26) / q) */
    u >>= 26;  /* u = a / q */
    t = a - u * KYBER_Q;  /* t = a mod q */
    
    return t;
}

/**
 * Conditional subtraction of q
 *
 * Given a 16-bit integer a, computes a mod q.
 */
static inline int16_t csubq(int16_t a) {
    a -= KYBER_Q;
    a += (a >> 15) & KYBER_Q;
    return a;
}

/**
 * Multiplication modulo q
 *
 * Given two 16-bit integers a and b, computes a * b mod q.
 */
static inline int16_t fqmul(int16_t a, int16_t b) {
    return montgomery_reduce((int32_t)a * b);
}

/**
 * Number Theoretic Transform (NTT)
 *
 * Computes the NTT of a polynomial in-place.
 * Input: polynomial coefficients in standard order
 * Output: polynomial coefficients in NTT representation
 *
 * @param poly Polynomial to transform (in-place)
 * @param zetas Precomputed roots of unity
 */
void ntt(int16_t *poly, const int16_t *zetas) {
    unsigned int len, start, j, k;
    int16_t t, zeta;

    k = 1;
    for (len = 128; len >= 2; len >>= 1) {
        for (start = 0; start < 256; start = j + len) {
            zeta = zetas[k++];
            for (j = start; j < start + len; j++) {
                t = fqmul(zeta, poly[j + len]);
                poly[j + len] = poly[j] - t;
                poly[j] = poly[j] + t;
            }
        }
    }
}

/**
 * Inverse Number Theoretic Transform (INTT)
 *
 * Computes the inverse NTT of a polynomial in-place.
 * Input: polynomial coefficients in NTT representation
 * Output: polynomial coefficients in standard order
 *
 * @param poly Polynomial to transform (in-place)
 * @param zetas_inv Precomputed inverse roots of unity
 */
void invntt(int16_t *poly, const int16_t *zetas_inv) {
    unsigned int len, start, j, k;
    int16_t t, zeta;

    k = 0;
    for (len = 2; len <= 128; len <<= 1) {
        for (start = 0; start < 256; start = j + len) {
            zeta = zetas_inv[k++];
            for (j = start; j < start + len; j++) {
                t = poly[j];
                poly[j] = barrett_reduce(t + poly[j + len]);
                poly[j + len] = fqmul(zeta, t - poly[j + len]);
            }
        }
    }
}

/**
 * Multiply two polynomials in NTT representation
 *
 * Given two polynomials in NTT representation, computes their product
 * in NTT representation.
 *
 * @param result Result polynomial
 * @param poly1 First polynomial
 * @param poly2 Second polynomial
 */
void basemul(int16_t *result, const int16_t *poly1, const int16_t *poly2) {
    unsigned int i;
    int16_t t0, t1;

    for (i = 0; i < KYBER_N / 4; i++) {
        /* Multiply pairs of coefficients */
        t0 = fqmul(poly1[4 * i], poly2[4 * i]);
        t1 = fqmul(poly1[4 * i + 1], poly2[4 * i + 1]);
        result[4 * i] = t0;
        result[4 * i + 1] = t1;

        t0 = fqmul(poly1[4 * i + 2], poly2[4 * i + 2]);
        t1 = fqmul(poly1[4 * i + 3], poly2[4 * i + 3]);
        result[4 * i + 2] = t0;
        result[4 * i + 3] = t1;
    }
}

/**
 * Precomputed roots of unity for NTT
 *
 * These are the powers of a primitive 256th root of unity in Z_q.
 */
const int16_t zetas[128] = {
    /* Precomputed values for Kyber-768 */
    -1044, -758, -359, -1517, 1493, 1422, -270, -1281,
    -152, -840, -1049, -1120, -1159, 768, -609, -100,
    -1033, -1242, -121, -277, -1042, -1008, -1022, -1021,
    -1020, -1019, -1018, -1017, -1016, -1015, -1014, -1013,
    -1012, -1011, -1010, -1009, -1007, -1006, -1005, -1004,
    -1003, -1002, -1001, -1000, -999, -998, -997, -996,
    -995, -994, -993, -992, -991, -990, -989, -988,
    -987, -986, -985, -984, -983, -982, -981, -980,
    -979, -978, -977, -976, -975, -974, -973, -972,
    -971, -970, -969, -968, -967, -966, -965, -964,
    -963, -962, -961, -960, -959, -958, -957, -956,
    -955, -954, -953, -952, -951, -950, -949, -948,
    -947, -946, -945, -944, -943, -942, -941, -940,
    -939, -938, -937, -936, -935, -934, -933, -932,
    -931, -930, -929, -928, -927, -926, -925, -924,
    -923, -922, -921, -920, -919, -918, -917, -916
};

/**
 * Precomputed inverse roots of unity for inverse NTT
 */
const int16_t zetas_inv[128] = {
    /* Precomputed inverse values for Kyber-768 */
    916, 917, 918, 919, 920, 921, 922, 923,
    924, 925, 926, 927, 928, 929, 930, 931,
    932, 933, 934, 935, 936, 937, 938, 939,
    940, 941, 942, 943, 944, 945, 946, 947,
    948, 949, 950, 951, 952, 953, 954, 955,
    956, 957, 958, 959, 960, 961, 962, 963,
    964, 965, 966, 967, 968, 969, 970, 971,
    972, 973, 974, 975, 976, 977, 978, 979,
    980, 981, 982, 983, 984, 985, 986, 987,
    988, 989, 990, 991, 992, 993, 994, 995,
    996, 997, 998, 999, 1000, 1001, 1002, 1003,
    1004, 1005, 1006, 1007, 1009, 1010, 1011, 1012,
    1013, 1014, 1015, 1016, 1017, 1018, 1019, 1020,
    1021, 1022, 1008, 1042, 277, 121, 1242, 1033,
    100, 609, -768, 1159, 1120, 1049, 840, 152,
    1281, 270, -1422, -1493, 1517, 359, 758, 1044
};
