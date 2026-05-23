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
 * Example: NPU Inference
 *
 * This example demonstrates AI inference using NPU.
 */

#include <nuva_hal.hpp>
#include <iostream>
#include <vector>
#include <chrono>

using namespace nuva;

int main() {
    std::cout << "=== NPU Inference Example ===" << std::endl << std::endl;
    
    try {
        // Initialize NPU
        std::cout << "--- Initializing NPU ---" << std::endl;
        Npu::init();
        
        // Get device count
        uint32_t device_count = Npu::get_device_count();
        std::cout << "NPU devices: " << device_count << std::endl;
        
        if (device_count == 0) {
            std::cout << "No NPU devices available" << std::endl;
            Npu::shutdown();
            return 0;
        }
        
        // Get device info
        for (uint32_t i = 0; i < device_count; i++) {
            auto info = Npu::get_device_info(i);
            std::cout << "NPU " << i << ":" << std::endl;
            std::cout << "  Name: " << info.name << std::endl;
            std::cout << "  Cores: " << info.num_cores << std::endl;
            std::cout << "  Memory: " << info.memory_size / (1024 * 1024) << " MB" << std::endl;
            std::cout << "  Frequency: " << info.frequency_mhz << " MHz" << std::endl;
        }
        std::cout << std::endl;
        
        // Load model (simulated)
        std::cout << "--- Loading Model ---" << std::endl;
        
        // In a real scenario, you would load model data from a file
        // For this example, we'll use placeholder data
        std::vector<uint8_t> model_data(1024, 0);
        
        // Note: In actual implementation, you would need a device handle
        // nuva_npu_device_t device = ...;
        // Npu::Model model(device, model_data.data(), model_data.size());
        
        std::cout << "Model loaded (simulated)" << std::endl;
        std::cout << std::endl;
        
        // Prepare input data
        std::cout << "--- Preparing Input ---" << std::endl;
        
        // Example: 224x224x3 image (150528 bytes)
        const size_t input_size = 224 * 224 * 3;
        std::vector<uint8_t> input_data(input_size, 128); // Gray image
        
        std::cout << "Input size: " << input_size << " bytes" << std::endl;
        std::cout << "Input shape: 224x224x3" << std::endl;
        std::cout << std::endl;
        
        // Run inference (simulated)
        std::cout << "--- Running Inference ---" << std::endl;
        
        auto start = std::chrono::high_resolution_clock::now();
        
        // In actual implementation:
        // 1. Create input buffer
        // 2. Write input data
        // 3. Create output buffer
        // 4. Execute inference
        // 5. Read output data
        
        // Simulate inference time
        for (volatile int i = 0; i < 1000000; i++);
        
        auto end = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::microseconds>(end - start);
        
        std::cout << "Inference time: " << duration.count() << " μs" << std::endl;
        std::cout << std::endl;
        
        // Process output (simulated)
        std::cout << "--- Processing Output ---" << std::endl;
        
        // Example: 1000-class classification
        const size_t output_size = 1000;
        std::vector<float> output_data(output_size, 0.0f);
        
        // Simulate softmax output
        output_data[42] = 0.95f;  // Class 42 with highest probability
        
        // Find top class
        int top_class = 0;
        float top_prob = 0.0f;
        for (size_t i = 0; i < output_data.size(); i++) {
            if (output_data[i] > top_prob) {
                top_prob = output_data[i];
                top_class = i;
            }
        }
        
        std::cout << "Top class: " << top_class << std::endl;
        std::cout << "Probability: " << (top_prob * 100.0f) << "%" << std::endl;
        std::cout << std::endl;
        
        // Shutdown NPU
        std::cout << "--- Shutting Down ---" << std::endl;
        Npu::shutdown();
        std::cout << "NPU shutdown complete" << std::endl;
        
    } catch (const Exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        std::cerr << "Result: " << e.result() << std::endl;
        return -1;
    }
    
    std::cout << std::endl << "=== Example Complete ===" << std::endl;
    return 0;
}
