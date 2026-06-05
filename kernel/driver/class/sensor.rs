/*
 * Nuva OS - Kernel - Driver - Class - Sensor
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
 * Nuva OS - Kernel - Sensor Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for sensor devices (accelerometer, gyroscope, etc.).
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Sensor Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    /// Accelerometer
    Accelerometer = 0,
    /// Gyroscope
    Gyroscope = 1,
    /// Magnetometer
    Magnetometer = 2,
    /// Ambient light sensor
    Light = 3,
    /// Proximity sensor
    Proximity = 4,
    /// Temperature sensor
    Temperature = 5,
    /// Pressure sensor (barometer)
    Pressure = 6,
    /// Humidity sensor
    Humidity = 7,
    /// Gravity sensor
    Gravity = 8,
    /// Linear acceleration
    LinearAccel = 9,
    /// Rotation vector
    RotationVector = 10,
    /// Step counter
    StepCounter = 11,
    /// Heart rate monitor
    HeartRate = 12,
    /// GPS
    Gps = 13,
    /// Custom sensor
    Custom = 255,
}

/// Sensor Data
#[repr(C)]
pub union SensorData {
    /// 3-axis data (accelerometer, gyroscope, magnetometer)
    pub vec3: SensorVec3,
    /// Scalar data (temperature, pressure, etc.)
    pub scalar: f32,
    /// Light data (lux)
    pub light: f32,
    /// Proximity data (distance in cm)
    pub proximity: f32,
    /// GPS data
    pub gps: SensorGps,
}

/// 3-axis sensor data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SensorVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// GPS sensor data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SensorGps {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
    pub speed: f32,
    pub bearing: f32,
    pub accuracy: f32,
}

/// Sensor Event
#[repr(C)]
pub struct SensorEvent {
    /// Sensor type
    pub sensor_type: SensorType,
    /// Timestamp (nanoseconds)
    pub timestamp: u64,
    /// Sensor data
    pub data: SensorData,
    /// Accuracy (0-3, higher is better)
    pub accuracy: u8,
    /// Reserved
    pub reserved: [u8; 3],
}

/// Sensor Configuration
#[repr(C)]
pub struct SensorConfig {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Maximum delay (microseconds)
    pub max_delay: u32,
    /// Minimum delay (microseconds)
    pub min_delay: u32,
    /// Power consumption (micro-watts)
    pub power: u32,
    /// Resolution
    pub resolution: f32,
    /// Maximum range
    pub max_range: f32,
    /// FIFO size (events)
    pub fifo_size: u32,
    /// Wake-up sensor
    pub wake_up: bool,
}

impl Default for SensorConfig {
    fn default() -> Self {
        SensorConfig {
            sample_rate: 100,
            max_delay: 1000000,
            min_delay: 1000,
            power: 0,
            resolution: 0.0,
            max_range: 0.0,
            fifo_size: 0,
            wake_up: false,
        }
    }
}

/// Sensor Device Operations
pub struct SensorDeviceOps {
    /// Activate sensor
    pub activate: Option<unsafe extern "C" fn(*mut core::ffi::c_void, bool) -> i32>,
    /// Set sample rate
    pub set_rate: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Get sample rate
    pub get_rate: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> u32>,
    /// Set configuration
    pub set_config:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const SensorConfig) -> i32>,
    /// Get configuration
    pub get_config: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SensorConfig) -> i32>,
    /// Read sensor data
    pub read: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SensorEvent) -> i32>,
    /// Flush FIFO
    pub flush: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// Sensor ioctl commands
pub mod sensor_ioctl {
    /// Activate sensor
    pub const ACTIVATE: u32 = 0xA001;
    /// Deactivate sensor
    pub const DEACTIVATE: u32 = 0xA002;
    /// Set sample rate
    pub const SET_RATE: u32 = 0xA003;
    /// Get sample rate
    pub const GET_RATE: u32 = 0xA004;
    /// Set configuration
    pub const SET_CONFIG: u32 = 0xA005;
    /// Get configuration
    pub const GET_CONFIG: u32 = 0xA006;
    /// Flush FIFO
    pub const FLUSH: u32 = 0xA007;
    /// Get available sensors
    pub const GET_SENSORS: u32 = 0xA008;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_type_values() {
        assert_eq!(SensorType::Accelerometer as i32, 0);
        assert_eq!(SensorType::Gyroscope as i32, 1);
        assert_eq!(SensorType::Temperature as i32, 5);
    }

    #[test]
    fn test_sensor_config_default() {
        let config = SensorConfig::default();
        assert_eq!(config.sample_rate, 100);
        assert_eq!(config.max_delay, 1000000);
    }
}
