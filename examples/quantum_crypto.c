/*
 * Nuva OS
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

/*
 * Example: Quantum Cryptography
 *
 * This example demonstrates quantum-safe cryptography
 * using CRYSTALS-Kyber and CRYSTALS-Dilithium.
 */

#include <nuva_hal.h>
#include <stdio.h>
#include <string.h>

void print_hex(const char* label, const uint8_t* data, size_t size) {
    printf("%s: ", label);
    for (size_t i = 0; i < size && i < 16; i++) {
        printf("%02X", data[i]);
    }
    if (size > 16) {
        printf("...");
    }
    printf("\n");
}

int main() {
    printf("=== Quantum Cryptography Example ===\n\n");
    
    nuva_result_t result;
    
    // ===== CRYSTALS-Kyber Example =====
    printf("--- CRYSTALS-Kyber (Key Encapsulation) ---\n");
    
    // Initialize PQC provider
    nuva_pqc_t pqc;
    result = nuva_pqc_init(&pqc);
    
    if (result != NUVA_OK) {
        printf("Failed to initialize PQC: %d\n", result);
        return -1;
    }
    
    // Generate Kyber-768 key pair
    printf("Generating Kyber-768 key pair...\n");
    nuva_key_t public_key, secret_key;
    result = nuva_pqc_kyber_keygen(pqc, NUVA_KYBER_768, &public_key, &secret_key);
    
    if (result == NUVA_OK) {
        printf("  Key pair generated successfully\n");
        
        // Encapsulate
        printf("Encapsulating shared secret...\n");
        uint8_t shared_secret1[32];
        size_t shared_secret_size = 32;
        uint8_t ciphertext[1088]; // Kyber-768 ciphertext size
        size_t ciphertext_size = 1088;
        
        result = nuva_pqc_kyber_encapsulate(pqc, public_key,
                                            shared_secret1, &shared_secret_size,
                                            ciphertext, &ciphertext_size);
        
        if (result == NUVA_OK) {
            print_hex("  Shared Secret", shared_secret1, 32);
            print_hex("  Ciphertext", ciphertext, 16);
            printf("  Ciphertext size: %zu bytes\n", ciphertext_size);
            
            // Decapsulate
            printf("Decapsulating shared secret...\n");
            uint8_t shared_secret2[32];
            size_t shared_secret_size2 = 32;
            
            result = nuva_pqc_kyber_decapsulate(pqc, secret_key,
                                                ciphertext, ciphertext_size,
                                                shared_secret2, &shared_secret_size2);
            
            if (result == NUVA_OK) {
                print_hex("  Decapsulated Secret", shared_secret2, 32);
                
                // Verify
                if (memcmp(shared_secret1, shared_secret2, 32) == 0) {
                    printf("  ✓ Shared secrets match!\n");
                } else {
                    printf("  ✗ Shared secrets do NOT match!\n");
                }
            }
        }
        
        // Free keys
        nuva_key_free(public_key);
        nuva_key_free(secret_key);
    }
    printf("\n");
    
    // ===== CRYSTALS-Dilithium Example =====
    printf("--- CRYSTALS-Dilithium (Digital Signatures) ---\n");
    
    // Generate Dilithium-3 key pair
    printf("Generating Dilithium-3 key pair...\n");
    result = nuva_pqc_dilithium_keygen(pqc, NUVA_DILITHIUM_3, &public_key, &secret_key);
    
    if (result == NUVA_OK) {
        printf("  Key pair generated successfully\n");
        
        // Message to sign
        const char* message = "Hello, Quantum World!";
        size_t message_size = strlen(message);
        printf("  Message: \"%s\"\n", message);
        
        // Sign
        printf("Signing message...\n");
        uint8_t signature[3293]; // Dilithium-3 signature size
        size_t signature_size = 3293;
        
        result = nuva_pqc_dilithium_sign(pqc, secret_key,
                                         (const uint8_t*)message, message_size,
                                         signature, &signature_size);
        
        if (result == NUVA_OK) {
            print_hex("  Signature", signature, 16);
            printf("  Signature size: %zu bytes\n", signature_size);
            
            // Verify
            printf("Verifying signature...\n");
            bool valid;
            result = nuva_pqc_dilithium_verify(pqc, public_key,
                                               (const uint8_t*)message, message_size,
                                               signature, signature_size,
                                               &valid);
            
            if (result == NUVA_OK) {
                if (valid) {
                    printf("  ✓ Signature is valid!\n");
                } else {
                    printf("  ✗ Signature is INVALID!\n");
                }
            }
            
            // Test with wrong message
            printf("Testing with wrong message...\n");
            const char* wrong_message = "Wrong Message!";
            result = nuva_pqc_dilithium_verify(pqc, public_key,
                                               (const uint8_t*)wrong_message, strlen(wrong_message),
                                               signature, signature_size,
                                               &valid);
            
            if (result == NUVA_OK) {
                if (!valid) {
                    printf("  ✓ Correctly rejected wrong message!\n");
                } else {
                    printf("  ✗ Incorrectly accepted wrong message!\n");
                }
            }
        }
        
        // Free keys
        nuva_key_free(public_key);
        nuva_key_free(secret_key);
    }
    printf("\n");
    
    // ===== QRNG Example =====
    printf("--- Quantum Random Number Generator ---\n");
    
    nuva_qrng_t qrng;
    result = nuva_qrng_init(&qrng);
    
    if (result == NUVA_OK) {
        printf("QRNG initialized successfully\n");
        
        // Generate random bytes
        uint8_t random_bytes[32];
        result = nuva_qrng_generate(qrng, random_bytes, 32);
        
        if (result == NUVA_OK) {
            print_hex("  Random Bytes", random_bytes, 32);
            printf("  ✓ Generated 32 random bytes\n");
        }
    } else {
        printf("QRNG not available\n");
    }
    printf("\n");
    
    printf("=== Example Complete ===\n");
    return 0;
}
