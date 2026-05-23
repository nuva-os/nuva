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

//! DisplayDeviceDriverImplementation
/*!*/
//! Provides unified interface and example implementation for display devices.

use alloc::sync::Arc;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, Ordering};

use super::{DeviceOps, DeviceInfo, DeviceState, DeviceType, DriverError};

// ============================================================================
// DisplayDeviceConfig
// ============================================================================

/// Display mode
#[derive(Debug, Clone, Copy)]
pub struct DisplayMode {
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Refresh rate
    pub refresh_rate: u32,
    /// Color depth (bits)
    pub bpp: u32,
    /// FlagBit
    pub flags: u32,
}

/// DisplayDeviceConfig
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    /// Current mode
    pub current_mode: DisplayMode,
    /// Supported mode list
    pub supported_modes: [DisplayMode; 16],
    /// Number of modes
    pub num_modes: u32,
    /// FrameBufferAddress
    pub framebuffer_addr: u64,
    /// FrameBufferSize
    pub framebuffer_size: u64,
    /// Double buffer support
    pub double_buffer: bool,
    /// Hardware cursor support
    pub hw_cursor: bool,
}

// ============================================================================
// DisplayDeviceOperationInterface
// ============================================================================

/// Display device extended operation interface
pub trait DisplayOps: DeviceOps {
    /// GetDisplayConfig
    fn get_config(&self) -> &DisplayConfig;
    
    /// Set display mode
    fn set_mode(&mut self, mode: &DisplayMode) -> Result<(), DriverError>;
    
    /// GetFrameBuffer
    fn get_framebuffer(&self) -> Result<(*mut u8, usize), DriverError>;
    
    /// RefreshDisplay
    fn flush(&mut self) -> Result<(), DriverError>;
    
    /// SetBrightness
    fn set_brightness(&mut self, brightness: u32) -> Result<(), DriverError>;
    
    /// GetBrightness
    fn get_brightness(&self) -> Result<u32, DriverError>;
    
    /// Set backlight
    fn set_backlight(&mut self, enable: bool) -> Result<(), DriverError>;
    
    /// Set cursor position
    fn set_cursor_position(&mut self, x: u32, y: u32) -> Result<(), DriverError>;
    
    /// Set cursor visibility
    fn set_cursor_visible(&mut self, visible: bool) -> Result<(), DriverError>;
}

// ============================================================================
// GeneralDisplayDeviceImplementation
// ============================================================================

/// GeneralDisplayDevice
pub struct GenericDisplay {
    /// DeviceInfo
    info: DeviceInfo,
    /// DisplayConfig
    config: DisplayConfig,
    /// CurrentBrightness
    brightness: AtomicU32,
    /// Backlight state
    backlight_enabled: bool,
    /// Cursor position
    cursor_x: u32,
    cursor_y: u32,
    /// Cursor visibility
    cursor_visible: bool,
}

impl GenericDisplay {
    /// Create a new display device
    pub fn new(name: String, config: DisplayConfig) -> Self {
        Self {
            info: DeviceInfo {
                name,
                device_type: DeviceType::Display,
                id: super::DeviceId {
                    vendor_id: 0,
                    device_id: 0,
                    bus_type: 0,
                    bus_number: 0,
                },
                state: DeviceState::Uninitialized,
                driver_name: String::from("generic_display"),
                device_path: String::from("/dev/display0"),
                private_data_size: 0,
                flags: 0,
            },
            config,
            brightness: AtomicU32::new(100),
            backlight_enabled: true,
            cursor_x: 0,
            cursor_y: 0,
            cursor_visible: true,
        }
    }
}

impl DeviceOps for GenericDisplay {
    fn get_info(&self) -> &DeviceInfo {
        &self.info
    }
    
    fn init(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Initialized;
        Ok(())
    }
    
    fn deinit(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Uninitialized;
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), DriverError> {
        if self.info.state != DeviceState::Initialized {
            return Err(DriverError::NotInitialized);
        }
        self.info.state = DeviceState::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Initialized;
        Ok(())
    }
    
    fn suspend(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Suspended;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Running;
        Ok(())
    }
    
    fn read(&self, buffer: &mut [u8], offset: u64) -> Result<usize, DriverError> {
        // Display device usually does not support read
        Err(DriverError::NotSupported)
    }
    
    fn write(&self, buffer: &[u8], offset: u64) -> Result<usize, DriverError> {
        // Write framebuffer
        // TODO: Implement framebuffer write
        Ok(buffer.len())
    }
    
    fn ioctl(&mut self, cmd: u32, arg: u64) -> Result<u64, DriverError> {
        match cmd {
            0 => self.set_brightness(arg as u32).map(|_| 0),
            1 => self.get_brightness().map(|b| b as u64),
            _ => Err(DriverError::NotSupported),
        }
    }
    
    fn get_state(&self) -> DeviceState {
        self.info.state
    }
    
    fn set_state(&mut self, state: DeviceState) {
        self.info.state = state;
    }
}

impl DisplayOps for GenericDisplay {
    fn get_config(&self) -> &DisplayConfig {
        &self.config
    }
    
    fn set_mode(&mut self, mode: &DisplayMode) -> Result<(), DriverError> {
        // Check if mode is supported
        for i in 0..self.config.num_modes as usize {
            if self.config.supported_modes[i].width == mode.width
                && self.config.supported_modes[i].height == mode.height
                && self.config.supported_modes[i].refresh_rate == mode.refresh_rate
            {
                self.config.current_mode = *mode;
                return Ok(());
            }
        }
        
        Err(DriverError::InvalidArgument)
    }
    
    fn get_framebuffer(&self) -> Result<(*mut u8, usize), DriverError> {
        if self.config.framebuffer_addr == 0 {
            return Err(DriverError::NotInitialized);
        }
        
        Ok((
            self.config.framebuffer_addr as *mut u8,
            self.config.framebuffer_size as usize,
        ))
    }
    
    fn flush(&mut self) -> Result<(), DriverError> {
        // TODO: Implement display refresh
        Ok(())
    }
    
    fn set_brightness(&mut self, brightness: u32) -> Result<(), DriverError> {
        if brightness > 100 {
            return Err(DriverError::InvalidArgument);
        }
        self.brightness.store(brightness, Ordering::Release);
        Ok(())
    }
    
    fn get_brightness(&self) -> Result<u32, DriverError> {
        Ok(self.brightness.load(Ordering::Acquire))
    }
    
    fn set_backlight(&mut self, enable: bool) -> Result<(), DriverError> {
        self.backlight_enabled = enable;
        Ok(())
    }
    
    fn set_cursor_position(&mut self, x: u32, y: u32) -> Result<(), DriverError> {
        if !self.config.hw_cursor {
            return Err(DriverError::NotSupported);
        }
        
        self.cursor_x = x;
        self.cursor_y = y;
        Ok(())
    }
    
    fn set_cursor_visible(&mut self, visible: bool) -> Result<(), DriverError> {
        if !self.config.hw_cursor {
            return Err(DriverError::NotSupported);
        }
        
        self.cursor_visible = visible;
        Ok(())
    }
}

// ============================================================================
// Display device driver plugin
// ============================================================================

/// Display device driver plugin
pub struct DisplayDriverPlugin {
    name: String,
    version: String,
}

impl DisplayDriverPlugin {
    pub fn new() -> Self {
        Self {
            name: String::from("display_driver"),
            version: String::from("1.0.0"),
        }
    }
}

impl super::DriverPlugin for DisplayDriverPlugin {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_version(&self) -> &str {
        &self.version
    }
    
    fn get_supported_types(&self) -> &[DeviceType] {
        &[DeviceType::Display]
    }
    
    fn probe(&self, device_info: &DeviceInfo) -> Result<bool, DriverError> {
        Ok(device_info.device_type == DeviceType::Display)
    }
    
    fn create_device(&self, device_info: &DeviceInfo) -> Result<Arc<dyn DeviceOps>, DriverError> {
        // Create default config
        let config = DisplayConfig {
            current_mode: DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                bpp: 32,
                flags: 0,
            },
            supported_modes: [DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                bpp: 32,
                flags: 0,
            }; 16],
            num_modes: 1,
            framebuffer_addr: 0,
            framebuffer_size: 1920 * 1080 * 4,
            double_buffer: false,
            hw_cursor: true,
        };
        
        let display = GenericDisplay::new(device_info.name.clone(), config);
        Ok(Arc::new(display))
    }
    
    fn destroy_device(&self, _device: &Arc<dyn DeviceOps>) -> Result<(), DriverError> {
        Ok(())
    }
    
    fn init(&mut self) -> Result<(), DriverError> {
        Ok(())
    }
    
    fn deinit(&mut self) -> Result<(), DriverError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_device() {
        let config = DisplayConfig {
            current_mode: DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                bpp: 32,
                flags: 0,
            },
            supported_modes: [DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                bpp: 32,
                flags: 0,
            }; 16],
            num_modes: 1,
            framebuffer_addr: 0x10000000,
            framebuffer_size: 1920 * 1080 * 4,
            double_buffer: false,
            hw_cursor: true,
        };
        
        let mut display = GenericDisplay::new(String::from("test_display"), config);
        
        assert_eq!(display.init(), Ok(()));
        assert_eq!(display.start(), Ok(()));
        assert_eq!(display.get_brightness(), Ok(100));
        assert_eq!(display.set_brightness(50), Ok(()));
        assert_eq!(display.get_brightness(), Ok(50));
    }
}