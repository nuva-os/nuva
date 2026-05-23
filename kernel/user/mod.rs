/*
 * Nuva OS - UserModule
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

pub mod user;
pub mod session;
pub mod task;
pub mod permission;

/// InitializeUserModule
pub fn init_user_module() {
    // InitializeUsermanagementadministration
    user::init_user_manager();
    
    // InitializeSessionmanagementadministration
    session::init_session_manager();
    
    // InitializeTask Management
    task::init_task_manager();
    
    // InitializePermissionSystem
    permission::init_permission();
    
    log_info!("User module initialized");
    log_info!("  Multi-user: enabled");
    log_info!("  Multi-task: enabled");
}