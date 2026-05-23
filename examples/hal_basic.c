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
 * Example: Basic HAL Usage
 *
 * This example demonstrates basic HAL API usage
 * for getting system information.
 */

#include <nuva_hal.h>
#include <stdio.h>

int main() {
    printf("=== Nuva OS HAL Example ===\n\n");
    
    // Get HAL version
    uint32_t version = nuva_hal_get_version();
    const char* version_str = nuva_hal_get_version_string();
    printf("HAL Version: %s (0x%08X)\n\n", version_str, version);
    
    // Get CPU information
    printf("--- CPU Information ---\n");
    nuva_cpu_info_t cpu_info;
    nuva_result_t result = nuva_cpu_get_info(&cpu_info);
    
    if (result == NUVA_OK) {
        printf("  Cores:         %u\n", cpu_info.core_count);
        printf("  Frequency:     %u MHz\n", cpu_info.frequency_mhz);
        printf("  Cache Line:    %u bytes\n", cpu_info.cache_line_size);
        printf("  Total Memory:  %lu MB\n", cpu_info.total_memory / (1024 * 1024));
        printf("  Vendor:        %.32s\n", cpu_info.vendor);
        printf("  Model:         %.64s\n", cpu_info.model);
    } else {
        printf("  Error: Failed to get CPU info (%d)\n", result);
    }
    printf("\n");
    
    // Get current core ID
    uint32_t core_id = nuva_cpu_get_core_id();
    printf("  Current Core:  %u\n\n", core_id);
    
    // Initialize GPU
    printf("--- GPU Information ---\n");
    result = nuva_gpu_init();
    
    if (result == NUVA_OK) {
        uint32_t gpu_count;
        result = nuva_gpu_get_device_count(&gpu_count);
        
        if (result == NUVA_OK) {
            printf("  GPU Count:     %u\n", gpu_count);
            
            for (uint32_t i = 0; i < gpu_count; i++) {
                nuva_gpu_info_t gpu_info;
                result = nuva_gpu_get_device_info(i, &gpu_info);
                
                if (result == NUVA_OK) {
                    printf("  GPU %u:\n", i);
                    printf("    Name:        %.64s\n", gpu_info.name);
                    printf("    Device ID:   0x%04X\n", gpu_info.device_id);
                    printf("    Vendor ID:   0x%04X\n", gpu_info.vendor_id);
                    printf("    Memory:      %lu MB\n", gpu_info.memory_size / (1024 * 1024));
                    printf("    Compute:     %u units\n", gpu_info.compute_units);
                }
            }
        }
        
        nuva_gpu_shutdown();
    } else {
        printf("  GPU not available\n");
    }
    printf("\n");
    
    // Initialize NPU
    printf("--- NPU Information ---\n");
    result = nuva_npu_init();
    
    if (result == NUVA_OK) {
        uint32_t npu_count;
        result = nuva_npu_get_device_count(&npu_count);
        
        if (result == NUVA_OK) {
            printf("  NPU Count:     %u\n", npu_count);
            
            for (uint32_t i = 0; i < npu_count; i++) {
                nuva_npu_info_t npu_info;
                result = nuva_npu_get_device_info(i, &npu_info);
                
                if (result == NUVA_OK) {
                    printf("  NPU %u:\n", i);
                    printf("    Name:        %.64s\n", npu_info.name);
                    printf("    Device ID:   0x%04X\n", npu_info.device_id);
                    printf("    Vendor ID:   0x%04X\n", npu_info.vendor_id);
                    printf("    Memory:      %lu MB\n", npu_info.memory_size / (1024 * 1024));
                    printf("    Cores:       %u\n", npu_info.num_cores);
                    printf("    Frequency:   %u MHz\n", npu_info.frequency_mhz);
                }
            }
        }
        
        nuva_npu_shutdown();
    } else {
        printf("  NPU not available\n");
    }
    printf("\n");
    
    printf("=== Example Complete ===\n");
    return 0;
}
