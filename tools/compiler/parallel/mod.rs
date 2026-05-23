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

// ! ParallelencodingtranslateModule
/*!*/
// ! SupportmanykernelParallelencodingtranslate, highencodingtranslatespeed

pub mod scheduler;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

/// Parallelencodingtranslatedevice
pub struct ParallelCompiler {
 /// Paralleltaskservicenumber
 jobs: usize,
 /// taskserviceschedulingdevice
 scheduler: scheduler::TaskScheduler,
}

impl ParallelCompiler {
 pub fn new(jobs: usize) -> Self {
 Self {
 jobs,
 scheduler: scheduler::TaskScheduler::new(jobs),
 }
 }

 /// ParallelencodingtranslatemanyitemsourceFile
 pub fn compile(&self, sources: &[PathBuf]) -> Result<Vec<CompileOutput>, ParallelError> {
 // createencodingtranslatetaskservice
 let tasks: Vec<CompileTask> = sources.iter()
 .map(|s| CompileTask {
 source: s.clone(),
 output: s.with_extension("o"),
 })
 .collect();

 // schedulingexecute
 let results = self.scheduler.execute(tasks)?;

 Ok(results)
 }

 /// GetParalleltaskservicenumber
 pub fn jobs(&self) -> usize {
 self.jobs
 }
}

impl Default for ParallelCompiler {
 fn default() -> Self {
 Self::new(num_cpus::get())
 }
}

/// encodingtranslatetaskservice
#[derive(Debug, Clone)]
pub struct CompileTask {
 pub source: PathBuf,
 pub output: PathBuf,
}

/// encodingtranslateoutput
#[derive(Debug, Clone)]
pub struct CompileOutput {
 pub source: PathBuf,
 pub output: PathBuf,
 pub success: bool,
 pub duration_ms: u64,
}

/// ParallelCompilation error
#[derive(Debug)]
pub enum ParallelError {
 TaskFailed(String),
 ThreadPanic,
}

impl std::fmt::Display for ParallelError {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 match self {
 ParallelError::TaskFailed(msg) => write!(f, "Task failed: {}", msg),
 ParallelError::ThreadPanic => write!(f, "Thread panic"),
 }
 }
}

impl std::error::Error for ParallelError {}