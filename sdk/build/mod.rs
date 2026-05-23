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

// ! buildsystemmodule

pub mod config;
pub mod target;
pub mod cache;
pub mod scheduler;
pub mod executor;
pub mod cross;

use crate::error::SdkError;

/// buildsystem
pub struct BuildSystem {
    /// buildconfigure
    config: config::BuildConfig,
    /// buildcache
    cache: cache::BuildCache,
    /// buildschedulingdevice
    scheduler: scheduler::BuildScheduler,
}

impl BuildSystem {
    pub fn new(config: config::BuildConfig) -> Self {
        Self {
            cache: cache::BuildCache::new(&config),
            scheduler: scheduler::BuildScheduler::new(&config),
            config,
        }
    }

    /// executebuild
    pub fn build(&mut self, target: &str) -> Result<BuildResult, SdkError> {
        // 1. parsebuildtarget
        let build_target = self.config.get_target(target)
            .ok_or_else(|| SdkError::BuildError(format!("Target not found: {}", target)))?;
        
        // 2. checkcache
        if self.cache.is_up_to_date(&build_target) {
            return Ok(BuildResult::Cached);
        }
        
        // 3. builddependencydiagram
        let dep_graph = self.scheduler.build_dependency_graph(&build_target)?;
        
        // 4. executebuild
        let result = self.scheduler.execute(&dep_graph)?;
        
        // 5. updatecache
        self.cache.update(&build_target)?;
        
        Ok(result)
    }

    /// clearadministrationbuildproductobject
    pub fn clean(&mut self, all: bool) -> Result<(), SdkError> {
        if all {
            self.cache.clear_all()?;
        } else {
            self.cache.clear()?;
        }
        Ok(())
    }

    /// getconfigure
    pub fn config(&self) -> &config::BuildConfig {
        &self.config
    }
}

/// buildresult
#[derive(Debug)]
pub enum BuildResult {
    /// succeed
    Success {
        /// outputfile
        outputs: Vec<std::path::PathBuf>,
        /// compiletime（millisecond）
        compile_time_ms: u64,
    },
    /// secondarycacheload
    Cached,
    /// fail
    Failed {
        /// errorinformation
        errors: Vec<String>,
    },
}