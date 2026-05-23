/*
 * Nuva OS HAL C++ API
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This header provides C++ wrapper for HAL C API
 * with RAII and exception safety.
 */

#ifndef NUVA_HAL_HPP
#define NUVA_HAL_HPP

#include "nuva_hal.h"
#include <stdexcept>
#include <memory>
#include <string>
#include <vector>

namespace nuva {

/*
 * Exception class
 */
class Exception : public std::runtime_error {
public:
    explicit Exception(nuva_result_t result)
        : std::runtime_error(error_message(result)), result_(result) {}

    nuva_result_t result() const { return result_; }

private:
    static std::string error_message(nuva_result_t result) {
        switch (result) {
            case NUVA_OK: return "Success";
            case NUVA_ERROR_INVALID_PARAM: return "Invalid parameter";
            case NUVA_ERROR_NOT_FOUND: return "Not found";
            case NUVA_ERROR_OUT_OF_MEMORY: return "Out of memory";
            case NUVA_ERROR_NOT_SUPPORTED: return "Not supported";
            case NUVA_ERROR_HARDWARE: return "Hardware error";
            case NUVA_ERROR_TIMEOUT: return "Timeout";
            case NUVA_ERROR_BUSY: return "Busy";
            default: return "Unknown error";
        }
    }

    nuva_result_t result_;
};

/*
 * Result checker
 */
inline void check_result(nuva_result_t result) {
    if (result != NUVA_OK) {
        throw Exception(result);
    }
}

/*
 * CPU HAL
 */
class Cpu {
public:
    struct Info {
        uint32_t core_count;
        uint32_t frequency_mhz;
        uint32_t cache_line_size;
        uint64_t total_memory;
        std::string vendor;
        std::string model;
    };

    static Info get_info() {
        nuva_cpu_info_t info;
        check_result(nuva_cpu_get_info(&info));
        
        return Info {
            info.core_count,
            info.frequency_mhz,
            info.cache_line_size,
            info.total_memory,
            std::string(info.vendor, strnlen(info.vendor, 32)),
            std::string(info.model, strnlen(info.model, 64)),
        };
    }

    static uint32_t get_core_id() {
        return nuva_cpu_get_core_id();
    }

    static void enable_irq() { nuva_cpu_enable_irq(); }
    static void disable_irq() { nuva_cpu_disable_irq(); }
    static void memory_barrier() { nuva_cpu_memory_barrier(); }
    static void read_barrier() { nuva_cpu_read_barrier(); }
    static void write_barrier() { nuva_cpu_write_barrier(); }
};

/*
 * GPU HAL
 */
class Gpu {
public:
    struct Info {
        uint32_t device_id;
        uint32_t vendor_id;
        uint64_t memory_size;
        uint32_t compute_units;
        std::string name;
    };

    class Buffer {
    public:
        Buffer(nuva_gpu_device_t device, size_t size) {
            check_result(nuva_gpu_create_buffer(device, size, &handle_));
        }

        ~Buffer() {
            if (handle_ != NUVA_INVALID_HANDLE) {
                nuva_gpu_destroy_buffer(handle_);
            }
        }

        Buffer(const Buffer&) = delete;
        Buffer& operator=(const Buffer&) = delete;

        Buffer(Buffer&& other) noexcept : handle_(other.handle_) {
            other.handle_ = NUVA_INVALID_HANDLE;
        }

        Buffer& operator=(Buffer&& other) noexcept {
            if (this != &other) {
                if (handle_ != NUVA_INVALID_HANDLE) {
                    nuva_gpu_destroy_buffer(handle_);
                }
                handle_ = other.handle_;
                other.handle_ = NUVA_INVALID_HANDLE;
            }
            return *this;
        }

        nuva_gpu_buffer_t handle() const { return handle_; }

    private:
        nuva_gpu_buffer_t handle_ = NUVA_INVALID_HANDLE;
    };

    static void init() { check_result(nuva_gpu_init()); }
    static void shutdown() { check_result(nuva_gpu_shutdown()); }

    static uint32_t get_device_count() {
        uint32_t count;
        check_result(nuva_gpu_get_device_count(&count));
        return count;
    }

    static Info get_device_info(uint32_t device_index) {
        nuva_gpu_info_t info;
        check_result(nuva_gpu_get_device_info(device_index, &info));
        
        return Info {
            info.device_id,
            info.vendor_id,
            info.memory_size,
            info.compute_units,
            std::string(info.name, strnlen(info.name, 64)),
        };
    }
};

/*
 * NPU HAL
 */
class Npu {
public:
    struct Info {
        uint32_t device_id;
        uint32_t vendor_id;
        uint64_t memory_size;
        uint32_t num_cores;
        uint32_t frequency_mhz;
        std::string name;
    };

    class Model {
    public:
        Model(nuva_npu_device_t device, const void* data, size_t size) {
            check_result(nuva_npu_load_model(device, data, size, &handle_));
        }

        ~Model() {
            if (handle_ != NUVA_INVALID_HANDLE) {
                nuva_npu_unload_model(handle_);
            }
        }

        Model(const Model&) = delete;
        Model& operator=(const Model&) = delete;

        Model(Model&& other) noexcept : handle_(other.handle_) {
            other.handle_ = NUVA_INVALID_HANDLE;
        }

        Model& operator=(Model&& other) noexcept {
            if (this != &other) {
                if (handle_ != NUVA_INVALID_HANDLE) {
                    nuva_npu_unload_model(handle_);
                }
                handle_ = other.handle_;
                other.handle_ = NUVA_INVALID_HANDLE;
            }
            return *this;
        }

        nuva_npu_model_t handle() const { return handle_; }

    private:
        nuva_npu_model_t handle_ = NUVA_INVALID_HANDLE;
    };

    class Buffer {
    public:
        Buffer(nuva_npu_device_t device, size_t size) {
            check_result(nuva_npu_create_buffer(device, size, &handle_));
        }

        ~Buffer() {
            if (handle_ != NUVA_INVALID_HANDLE) {
                nuva_npu_destroy_buffer(handle_);
            }
        }

        Buffer(const Buffer&) = delete;
        Buffer& operator=(const Buffer&) = delete;

        Buffer(Buffer&& other) noexcept : handle_(other.handle_) {
            other.handle_ = NUVA_INVALID_HANDLE;
        }

        Buffer& operator=(Buffer&& other) noexcept {
            if (this != &other) {
                if (handle_ != NUVA_INVALID_HANDLE) {
                    nuva_npu_destroy_buffer(handle_);
                }
                handle_ = other.handle_;
                other.handle_ = NUVA_INVALID_HANDLE;
            }
            return *this;
        }

        void write(const void* data, size_t size) {
            check_result(nuva_npu_write_buffer(handle_, data, size));
        }

        void read(void* data, size_t size) {
            check_result(nuva_npu_read_buffer(handle_, data, size));
        }

        nuva_npu_buffer_t handle() const { return handle_; }

    private:
        nuva_npu_buffer_t handle_ = NUVA_INVALID_HANDLE;
    };

    static void init() { check_result(nuva_npu_init()); }
    static void shutdown() { check_result(nuva_npu_shutdown()); }

    static uint32_t get_device_count() {
        uint32_t count;
        check_result(nuva_npu_get_device_count(&count));
        return count;
    }

    static Info get_device_info(uint32_t device_index) {
        nuva_npu_info_t info;
        check_result(nuva_npu_get_device_info(device_index, &info));
        
        return Info {
            info.device_id,
            info.vendor_id,
            info.memory_size,
            info.num_cores,
            info.frequency_mhz,
            std::string(info.name, strnlen(info.name, 64)),
        };
    }

    static void execute(
        const Model& model,
        const std::vector<nuva_npu_buffer_t>& inputs,
        std::vector<nuva_npu_buffer_t>& outputs
    ) {
        check_result(nuva_npu_execute(
            model.handle(),
            inputs.data(),
            inputs.size(),
            outputs.data(),
            outputs.size()
        ));
    }
};

/*
 * Version
 */
inline uint32_t get_version() { return nuva_hal_get_version(); }
inline std::string get_version_string() { return nuva_hal_get_version_string(); }

} // namespace nuva

#endif /* NUVA_HAL_HPP */
