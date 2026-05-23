/*
 * Nuva OS HAL C API
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This header provides C-compatible interface for HAL
 * enabling C/C++ driver development.
 */

#ifndef NUVA_HAL_H
#define NUVA_HAL_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ===== Common Types ===== */

/**
 * HAL result codes
 */
typedef enum {
    NUVA_OK = 0,
    NUVA_ERROR_INVALID_PARAM = -1,
    NUVA_ERROR_NOT_FOUND = -2,
    NUVA_ERROR_OUT_OF_MEMORY = -3,
    NUVA_ERROR_NOT_SUPPORTED = -4,
    NUVA_ERROR_HARDWARE = -5,
    NUVA_ERROR_TIMEOUT = -6,
    NUVA_ERROR_BUSY = -7,
} nuva_result_t;

/**
 * Handle type for opaque objects
 */
typedef uint64_t nuva_handle_t;

#define NUVA_INVALID_HANDLE 0

/* ===== CPU HAL ===== */

/**
 * CPU information
 */
typedef struct {
    uint32_t core_count;
    uint32_t frequency_mhz;
    uint32_t cache_line_size;
    uint64_t total_memory;
    char vendor[32];
    char model[64];
} nuva_cpu_info_t;

/**
 * Get CPU information
 */
nuva_result_t nuva_cpu_get_info(nuva_cpu_info_t* info);

/**
 * Get current core ID
 */
uint32_t nuva_cpu_get_core_id(void);

/**
 * Enable/disable interrupts
 */
void nuva_cpu_enable_irq(void);
void nuva_cpu_disable_irq(void);

/**
 * Memory barriers
 */
void nuva_cpu_memory_barrier(void);
void nuva_cpu_read_barrier(void);
void nuva_cpu_write_barrier(void);

/* ===== GPU HAL ===== */

/**
 * GPU device handle
 */
typedef nuva_handle_t nuva_gpu_device_t;

/**
 * GPU buffer handle
 */
typedef nuva_handle_t nuva_gpu_buffer_t;

/**
 * GPU device information
 */
typedef struct {
    uint32_t device_id;
    uint32_t vendor_id;
    uint64_t memory_size;
    uint32_t compute_units;
    char name[64];
} nuva_gpu_info_t;

/**
 * Initialize GPU subsystem
 */
nuva_result_t nuva_gpu_init(void);

/**
 * Shutdown GPU subsystem
 */
nuva_result_t nuva_gpu_shutdown(void);

/**
 * Get GPU device count
 */
nuva_result_t nuva_gpu_get_device_count(uint32_t* count);

/**
 * Get GPU device info
 */
nuva_result_t nuva_gpu_get_device_info(uint32_t device_index, nuva_gpu_info_t* info);

/**
 * Create GPU buffer
 */
nuva_result_t nuva_gpu_create_buffer(nuva_gpu_device_t device, 
                                      size_t size, 
                                      nuva_gpu_buffer_t* buffer);

/**
 * Destroy GPU buffer
 */
nuva_result_t nuva_gpu_destroy_buffer(nuva_gpu_buffer_t buffer);

/* ===== NPU HAL ===== */

/**
 * NPU device handle
 */
typedef nuva_handle_t nuva_npu_device_t;

/**
 * NPU model handle
 */
typedef nuva_handle_t nuva_npu_model_t;

/**
 * NPU buffer handle
 */
typedef nuva_handle_t nuva_npu_buffer_t;

/**
 * NPU device information
 */
typedef struct {
    uint32_t device_id;
    uint32_t vendor_id;
    uint64_t memory_size;
    uint32_t num_cores;
    uint32_t frequency_mhz;
    char name[64];
} nuva_npu_info_t;

/**
 * Initialize NPU subsystem
 */
nuva_result_t nuva_npu_init(void);

/**
 * Shutdown NPU subsystem
 */
nuva_result_t nuva_npu_shutdown(void);

/**
 * Get NPU device count
 */
nuva_result_t nuva_npu_get_device_count(uint32_t* count);

/**
 * Get NPU device info
 */
nuva_result_t nuva_npu_get_device_info(uint32_t device_index, nuva_npu_info_t* info);

/**
 * Load model into NPU
 */
nuva_result_t nuva_npu_load_model(nuva_npu_device_t device,
                                   const void* model_data,
                                   size_t model_size,
                                   nuva_npu_model_t* model);

/**
 * Unload model from NPU
 */
nuva_result_t nuva_npu_unload_model(nuva_npu_model_t model);

/**
 * Create NPU buffer
 */
nuva_result_t nuva_npu_create_buffer(nuva_npu_device_t device,
                                      size_t size,
                                      nuva_npu_buffer_t* buffer);

/**
 * Destroy NPU buffer
 */
nuva_result_t nuva_npu_destroy_buffer(nuva_npu_buffer_t buffer);

/**
 * Write to NPU buffer
 */
nuva_result_t nuva_npu_write_buffer(nuva_npu_buffer_t buffer,
                                     const void* data,
                                     size_t size);

/**
 * Read from NPU buffer
 */
nuva_result_t nuva_npu_read_buffer(nuva_npu_buffer_t buffer,
                                    void* data,
                                    size_t size);

/**
 * Execute inference
 */
nuva_result_t nuva_npu_execute(nuva_npu_model_t model,
                                const nuva_npu_buffer_t* inputs,
                                uint32_t input_count,
                                nuva_npu_buffer_t* outputs,
                                uint32_t output_count);

/* ===== Quantum HAL ===== */

/**
 * QRNG provider handle
 */
typedef nuva_handle_t nuva_qrng_t;

/**
 * PQC provider handle
 */
typedef nuva_handle_t nuva_pqc_t;

/**
 * Key handle
 */
typedef nuva_handle_t nuva_key_t;

/**
 * Kyber variant
 */
typedef enum {
    NUVA_KYBER_512 = 0,
    NUVA_KYBER_768 = 1,
    NUVA_KYBER_1024 = 2,
} nuva_kyber_variant_t;

/**
 * Dilithium variant
 */
typedef enum {
    NUVA_DILITHIUM_2 = 0,
    NUVA_DILITHIUM_3 = 1,
    NUVA_DILITHIUM_5 = 2,
} nuva_dilithium_variant_t;

/**
 * Initialize QRNG
 */
nuva_result_t nuva_qrng_init(nuva_qrng_t* qrng);

/**
 * Generate random bytes
 */
nuva_result_t nuva_qrng_generate(nuva_qrng_t qrng, 
                                  uint8_t* buffer, 
                                  size_t size);

/**
 * Initialize PQC provider
 */
nuva_result_t nuva_pqc_init(nuva_pqc_t* pqc);

/**
 * Generate Kyber key pair
 */
nuva_result_t nuva_pqc_kyber_keygen(nuva_pqc_t pqc,
                                     nuva_kyber_variant_t variant,
                                     nuva_key_t* public_key,
                                     nuva_key_t* secret_key);

/**
 * Kyber encapsulate
 */
nuva_result_t nuva_pqc_kyber_encapsulate(nuva_pqc_t pqc,
                                          nuva_key_t public_key,
                                          uint8_t* shared_secret,
                                          size_t* shared_secret_size,
                                          uint8_t* ciphertext,
                                          size_t* ciphertext_size);

/**
 * Kyber decapsulate
 */
nuva_result_t nuva_pqc_kyber_decapsulate(nuva_pqc_t pqc,
                                          nuva_key_t secret_key,
                                          const uint8_t* ciphertext,
                                          size_t ciphertext_size,
                                          uint8_t* shared_secret,
                                          size_t* shared_secret_size);

/**
 * Generate Dilithium key pair
 */
nuva_result_t nuva_pqc_dilithium_keygen(nuva_pqc_t pqc,
                                         nuva_dilithium_variant_t variant,
                                         nuva_key_t* public_key,
                                         nuva_key_t* secret_key);

/**
 * Dilithium sign
 */
nuva_result_t nuva_pqc_dilithium_sign(nuva_pqc_t pqc,
                                       nuva_key_t secret_key,
                                       const uint8_t* message,
                                       size_t message_size,
                                       uint8_t* signature,
                                       size_t* signature_size);

/**
 * Dilithium verify
 */
nuva_result_t nuva_pqc_dilithium_verify(nuva_pqc_t pqc,
                                         nuva_key_t public_key,
                                         const uint8_t* message,
                                         size_t message_size,
                                         const uint8_t* signature,
                                         size_t signature_size,
                                         bool* valid);

/**
 * Free key
 */
nuva_result_t nuva_key_free(nuva_key_t key);

/* ===== Power HAL ===== */

/**
 * Power state
 */
typedef enum {
    NUVA_POWER_ON = 0,
    NUVA_POWER_SLEEP = 1,
    NUVA_POWER_SUSPEND = 2,
    NUVA_POWER_OFF = 3,
} nuva_power_state_t;

/**
 * Set device power state
 */
nuva_result_t nuva_power_set_state(nuva_handle_t device, 
                                    nuva_power_state_t state);

/**
 * Get device power state
 */
nuva_result_t nuva_power_get_state(nuva_handle_t device,
                                    nuva_power_state_t* state);

/* ===== Version Information ===== */

/**
 * Get HAL version
 */
uint32_t nuva_hal_get_version(void);

/**
 * Get HAL version string
 */
const char* nuva_hal_get_version_string(void);

#ifdef __cplusplus
}
#endif

#endif /* NUVA_HAL_H */
