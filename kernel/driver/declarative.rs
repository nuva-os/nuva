/*
 * Nuva OS - Kernel - Declarative Driver Model
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

use alloc::string::String;
use alloc::vec::Vec;

use crate::kernel::error::{KernelError, KernelResult};

/**
 * Declarative driver trait.
 *
 * Drivers implement this trait to describe their capabilities,
 * resource requirements, and lifecycle hooks using a declarative
 * paradigm. The driver framework automatically matches devices
 * from the device tree, allocates resources, and manages the
 * driver lifecycle.
 *
 * # Example
 * ```rust
 * struct MyDriver;
 *
 * impl DeclarativeDriver for MyDriver {
 *     fn descriptor() -> DriverDescriptor {
 *         DriverDescriptor {
 *             name: "my_driver",
 *             compatible: &["vendor,my-device"],
 *             resources: &[ResourceDescriptor::irq(42)],
 *             capabilities: CapabilityFlags::READ | CapabilityFlags::WRITE,
 *             ..DriverDescriptor::default()
 *         }
 *     }
 *
 *     fn probe(&mut self, device: &DeviceMatch) -> KernelResult<()> { ... }
 *     fn remove(&mut self) -> KernelResult<()> { ... }
 * }
 * ```
 */
pub trait DeclarativeDriver {
    /** Return the static driver descriptor */
    fn descriptor() -> DriverDescriptor;

    /** Probe the driver against a matched device */
    fn probe(&mut self, match_info: &DeviceMatch) -> KernelResult<()>;

    /** Remove the driver from the device */
    fn remove(&mut self) -> KernelResult<()>;

    /** Suspend the device (optional, default no-op) */
    fn suspend(&mut self) -> KernelResult<()> {
        Ok(())
    }

    /** Resume the device (optional, default no-op) */
    fn resume(&mut self) -> KernelResult<()> {
        Ok(())
    }
}

/**
 * Static driver descriptor for declarative registration.
 *
 * Contains all metadata needed for the driver framework to
 * match devices, allocate resources, and manage the driver.
 */
#[derive(Debug, Clone)]
pub struct DriverDescriptor {
    /** Driver name (unique identifier) */
    pub name: &'static str,

    /** Device tree compatible strings for matching */
    pub compatible: &'static [&'static str],

    /** Resource requirements */
    pub resources: &'static [ResourceDescriptor],

    /** Capability flags */
    pub capabilities: CapabilityFlags,

    /** Driver priority (lower = higher priority) */
    pub priority: u32,

    /** Whether this driver supports hotplug */
    pub hotplug: bool,
}

/**
 * Bitflags for driver capabilities.
 */
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CapabilityFlags: u32 {
        /** Device supports read operations */
        const READ = 0x01;
        /** Device supports write operations */
        const WRITE = 0x02;
        /** Device supports ioctl operations */
        const IOCTL = 0x04;
        /** Device supports mmap operations */
        const MMAP = 0x08;
        /** Device supports power management */
        const PM = 0x10;
        /** Device supports DMA */
        const DMA = 0x20;
        /** Device supports interrupt handling */
        const IRQ = 0x40;
        /** Device is a bus controller */
        const BUS = 0x80;
    }
}

/**
 * Resource descriptor for declarative resource allocation.
 */
#[derive(Debug, Clone, Copy)]
pub enum ResourceDescriptor {
    /** IRQ resource with number */
    Irq { number: u32 },
    /** Memory-mapped I/O region */
    Mmio { base: u64, size: u64 },
    /** Clock resource by name index */
    Clock { index: u32 },
    /** Power domain by index */
    PowerDomain { index: u32 },
    /** GPIO pin */
    Gpio { number: u32 },
    /** I2C bus address */
    I2c { bus: u32, addr: u16 },
    /** DMA channel */
    Dma { channel: u32 },
}

/**
 * Device match information provided to probe().
 *
 * Contains the device tree node and allocated resources
 * that the driver can use.
 */
#[derive(Debug, Clone)]
pub struct DeviceMatch {
    /** Matched compatible string */
    pub compatible: String,
    /** Device node path in device tree */
    pub path: String,
    /** Allocated resources */
    pub allocated_resources: Vec<AllocatedResource>,
}

/**
 * An allocated resource bound to a device.
 */
#[derive(Debug, Clone)]
pub struct AllocatedResource {
    /** Original descriptor */
    pub descriptor: ResourceDescriptor,
    /** Virtual address for MMIO regions */
    pub vaddr: Option<usize>,
    /** Physical address for MMIO regions */
    pub paddr: Option<u64>,
}

/**
 * Declarative driver registry.
 *
 * Maintains a list of registered driver descriptors and
 * matches them against device tree nodes during boot.
 */
pub struct DriverRegistry {
    /** Registered driver descriptors */
    drivers: Vec<DriverDescriptor>,
}

impl DriverRegistry {
    /** Create a new empty registry */
    pub const fn new() -> Self {
        DriverRegistry {
            drivers: Vec::new(),
        }
    }

    /** Register a driver descriptor */
    pub fn register(&mut self, desc: DriverDescriptor) -> KernelResult<()> {
        if desc.name.is_empty() {
            return Err(KernelError::InvalidArgument);
        }
        if desc.compatible.is_empty() {
            return Err(KernelError::InvalidArgument);
        }
        self.drivers.push(desc);
        Ok(())
    }

    /**
     * Find matching drivers for a compatible string.
     *
     * Returns drivers sorted by priority (lowest first).
     */
    pub fn find_matches(&self, compatible: &str) -> Vec<&DriverDescriptor> {
        let mut matches: Vec<&DriverDescriptor> = self
            .drivers
            .iter()
            .filter(|d| d.compatible.iter().any(|c| *c == compatible))
            .collect();
        matches.sort_by_key(|d| d.priority);
        matches
    }

    /** Get number of registered drivers */
    pub fn driver_count(&self) -> usize {
        self.drivers.len()
    }
}
