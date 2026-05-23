use crate::{pr_info, pr_warn};
/*
 * Nuva OS - Kernel - Services
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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// powerState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
 /// runinfix
 Running = 0,
 /// emptyidle
 Idle = 1,
 /// machine
 Standby = 2,
 /// suspendtoMemory (S3)
 Suspend = 3,
 /// suspendtomagneticdisk (S4)
 Hibernate = 4,
 /// softclosemachine (S5)
 SoftOff = 5,
 /// hardclosemachine
 HardOff = 6,
}

/// CPU powerState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuPowerState {
 /// C0: run
 C0 = 0,
 /// C1: Halt
 C1 = 1,
 /// C2: Stop-Clock
 C2 = 2,
 /// C3: Sleep
 C3 = 3,
 /// C4: Deeper Sleep
 C4 = 4,
 /// C5: C6
 C5 = 5,
 /// C6: mostdeep
 C6 = 6,
}

/// DevicepowerState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePowerState {
 /// D0: allrun
 D0 = 0,
 /// D1: lowWorkconsume
 D1 = 1,
 /// D2: updatelowWorkconsume
 D2 = 2,
 /// D3: Close
 D3 = 3,
 /// D3Cold: allbreakelectric
 D3Cold = 4,
}

/// powerEvent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
 /// powerButton
 PowerButton = 0,
 /// Button
 SleepButton = 1,
 /// coverChildClose
 LidClose = 2,
 /// coverChildOpen
 LidOpen = 3,
 /// electricpoollowelectricquantification
 BatteryLow = 4,
 /// electricpoolboundary
 BatteryCritical = 5,
 /// AC Join
 AcConnected = 6,
 /// AC Disconnect
 AcDisconnected = 7,
 /// Timer
 Timer = 8,
 /// Wake
 Wakeup = 9,
}

/// electricpoolState
pub struct BatteryStatus {
 /// ifexist
 pub present: bool,
 /// ifinelectric
 pub charging: bool,
 /// ifinreleaseelectric
 pub discharging: bool,
 /// electricquantificationhundredsplitratio
 pub percent: u32,
 /// capacity
 pub capacity: u32,
 /// Voltage
 pub voltage: u32,
 /// Electric Current
 pub current: u32,
 /// remainingremainderTime (second)
 pub remaining_time: u32,
}

impl BatteryStatus {
 pub const fn new() -> Self {
 BatteryStatus {
 present: false,
 charging: false,
 discharging: false,
 percent: 0,
 capacity: 0,
 voltage: 0,
 current: 0,
 remaining_time: 0,
 }
 }
}

/// powermanagementadministrationOperation
pub struct PowerOps {
 /// Enter
 pub suspend: Option<fn() -> i32>,
 /// EnterSuspend
 pub hibernate: Option<fn() -> i32>,
 /// Wake
 pub resume: Option<fn() -> i32>,
 /// closemachine
 pub shutdown: Option<fn()>,
 /// repeatstart
 pub reboot: Option<fn()>,
 /// Set CPU State
 pub set_cpu_state: Option<fn(u32, CpuPowerState) -> i32>,
 /// SetDeviceState
 pub set_device_state: Option<fn(u32, DevicePowerState) -> i32>,
}

/// powerManager
pub struct PowerManager {
 /// CurrentpowerState
 pub state: AtomicU32,
 /// CPU count
 pub nr_cpus: u32,
 /// electricpoolState
 pub battery: BatteryStatus,
 /// Operation
 pub ops: PowerOps,
 /// emptyidleTime (millisecond)
 pub idle_time: AtomicU64,
 /// emptyidleThreshold (millisecond)
 pub idle_threshold: AtomicU64,
 /// statistics: Entertimenumber
 pub suspend_count: AtomicU32,
 /// statistics: EnterSuspendtimenumber
 pub hibernate_count: AtomicU32,
 /// statistics: Waketimenumber
 pub wakeup_count: AtomicU32,
}

impl PowerManager {
 pub const fn new() -> Self {
 PowerManager {
 state: AtomicU32::new(PowerState::Running as u32),
 nr_cpus: 1,
 battery: BatteryStatus::new(),
 ops: PowerOps {
 suspend: None,
 hibernate: None,
 resume: None,
 shutdown: None,
 reboot: None,
 set_cpu_state: None,
 set_device_state: None,
 },
 idle_time: AtomicU64::new(0),
 idle_threshold: AtomicU64::new(30000), // 30 second
 suspend_count: AtomicU32::new(0),
 hibernate_count: AtomicU32::new(0),
 wakeup_count: AtomicU32::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&mut self, nr_cpus: u32) {
 self.nr_cpus = nr_cpus;
 
 log_info!("Power manager initialized");
 log_info!(" CPUs: {}", nr_cpus);
 log_info!(" Idle threshold: {} ms", self.idle_threshold.load(Ordering::Acquire));
 }
 
 /// GetCurrentState
 pub fn get_state(&self) -> PowerState {
 match self.state.load(Ordering::Acquire) {
 0 => PowerState::Running,
 1 => PowerState::Idle,
 2 => PowerState::Standby,
 3 => PowerState::Suspend,
 4 => PowerState::Hibernate,
 5 => PowerState::SoftOff,
 6 => PowerState::HardOff,
 _ => PowerState::Running,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: PowerState) {
 self.state.store(state as u32, Ordering::Release);
 }
 
 /// Enter (S3)
 pub fn suspend(&self) -> i32 {
 log_info!("Entering suspend (S3)...");
 
 self.suspend_count.fetch_add(1, Ordering::AcqRel);
 self.set_state(PowerState::Suspend);
 
 if let Some(suspend) = self.ops.suspend {
 suspend()
 } else {
 -1
 }
 }
 
 /// EnterSuspend (S4)
 pub fn hibernate(&self) -> i32 {
 log_info!("Entering hibernate (S4)...");
 
 self.hibernate_count.fetch_add(1, Ordering::AcqRel);
 self.set_state(PowerState::Hibernate);
 
 if let Some(hibernate) = self.ops.hibernate {
 hibernate()
 } else {
 -1
 }
 }
 
 /// Wake
 pub fn resume(&self) -> i32 {
 log_info!("Resuming from sleep...");
 
 self.wakeup_count.fetch_add(1, Ordering::AcqRel);
 self.set_state(PowerState::Running);
 
 if let Some(resume) = self.ops.resume {
 resume()
 } else {
 0
 }
 }
 
 /// closemachine
 pub fn shutdown(&self) {
 log_info!("Shutting down...");
 
 self.set_state(PowerState::SoftOff);
 
 if let Some(shutdown) = self.ops.shutdown {
 shutdown();
 }
 }
 
 /// repeatstart
 pub fn reboot(&self) {
 log_info!("Rebooting...");
 
 if let Some(reboot) = self.ops.reboot {
 reboot();
 }
 }
 
 /// Set CPU powerState
 pub fn set_cpu_state(&self, cpu: u32, state: CpuPowerState) -> i32 {
 if let Some(set_state) = self.ops.set_cpu_state {
 set_state(cpu, state)
 } else {
 0
 }
 }
 
 /// SetDevicepowerState
 pub fn set_device_state(&self, device: u32, state: DevicePowerState) -> i32 {
 if let Some(set_state) = self.ops.set_device_state {
 set_state(device, state)
 } else {
 0
 }
 }
 
 /// UpdateemptyidleTime
 pub fn update_idle_time(&self, time_ms: u64) {
 self.idle_time.store(time_ms, Ordering::Release);
 
 // Checkifneedwantselfdynamic
 let threshold = self.idle_threshold.load(Ordering::Acquire);
 if time_ms >= threshold && self.get_state() == PowerState::Running {
 // selfdynamicEnteremptyidleState
 self.set_state(PowerState::Idle);
 }
 }
 
 /// SetemptyidleThreshold
 pub fn set_idle_threshold(&self, threshold_ms: u64) {
 self.idle_threshold.store(threshold_ms, Ordering::Release);
 }
 
 /// HandlepowerEvent
 pub fn handle_event(&self, event: PowerEvent) {
 match event {
 PowerEvent::PowerButton => {
 // powerButton: Displayclosemachinelogframeordirectacceptclosemachine
 self.shutdown();
 }
 PowerEvent::SleepButton => {
 // Button: Enter
 self.suspend();
 }
 PowerEvent::LidClose => {
 // coverChildClose: Enter
 self.suspend();
 }
 PowerEvent::LidOpen => {
 // coverChildOpen: Wake
 if self.get_state() == PowerState::Suspend {
 self.resume();
 }
 }
 PowerEvent::BatteryLow => {
 // lowelectricquantification: Warning
 log_warn!("Battery low!");
 }
 PowerEvent::BatteryCritical => {
 // boundaryelectricquantification: ForceSuspend
 log_warn!("Battery critical! Entering hibernate...");
 self.hibernate();
 }
 PowerEvent::AcConnected => {
 // AC Join
 log_info!("AC power connected");
 }
 PowerEvent::AcDisconnected => {
 // AC Disconnect
 log_info!("AC power disconnected");
 }
 PowerEvent::Timer => {
 // TimerEvent
 }
 PowerEvent::Wakeup => {
 // WakeEvent
 self.resume();
 }
 }
 }
 
 /// GetelectricpoolState
 pub fn get_battery_status(&self) -> &BatteryStatus {
 &self.battery
 }
 
 /// printstatisticsInfo
 pub fn print_stats(&self) {
 log_info!("Power Manager Statistics:");
 log_info!(" Current state: {:?}", self.get_state());
 log_info!(" Suspend count: {}", self.suspend_count.load(Ordering::Acquire));
 log_info!(" Hibernate count: {}", self.hibernate_count.load(Ordering::Acquire));
 log_info!(" Wakeup count: {}", self.wakeup_count.load(Ordering::Acquire));
 log_info!(" Idle time: {} ms", self.idle_time.load(Ordering::Acquire));
 }
}

/// GlobalpowerManager
static POWER_MANAGER: core::sync::OnceLock<PowerManager> = core::sync::OnceLock::new();

pub fn get_power_manager() -> &'static mut PowerManager {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut POWER_MANAGER }
}

pub fn init_power_manager(nr_cpus: u32) {
 let pm = get_power_manager();
 pm.init(nr_cpus);
}