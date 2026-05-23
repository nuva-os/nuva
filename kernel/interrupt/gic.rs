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

//! GIC stub module

use crate::kernel::driver::irq::IrqHandler;
use crate::pr_info;

pub struct GicController;

impl GicController {
    pub fn set_irq_type(&mut self, _irq: u32, _edge: bool) {
        log_info!("GIC: Set IRQ {} type edge={}", _irq, _edge);
    }
}

pub fn init_gic() {
    log_info!("GIC: Initializing GIC");
}
pub fn enable_irq(_irq: u32) {
    log_info!("GIC: Enable IRQ {}", _irq);
}
pub fn disable_irq(_irq: u32) {
    log_info!("GIC: Disable IRQ {}", _irq);
}
pub fn register_irq(_irq: u32, _handler: IrqHandler, _arg: *mut crate::kernel::driver::irq::IrqContext) -> bool {
    log_info!("GIC: Register IRQ {}", _irq);
    true
}
pub fn get_gic() -> Option<&'static mut GicController> {
    log_info!("GIC: Get GIC controller");
    None
}
pub fn gic_handle_irq() -> u32 {
    log_info!("GIC: Handle IRQ");
    0
}
