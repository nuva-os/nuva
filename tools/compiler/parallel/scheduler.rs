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

// ! taskserviceschedulingdevice

use std::path::PathBuf;
use std::sync::{Arc, Mutex, Condvar};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;
use std::time::Instant;

use super::{CompileTask, CompileOutput, ParallelError};

/// taskserviceschedulingdevice
pub struct TaskScheduler {
 /// workmakeThreadnumber
 workers: usize,
}

impl TaskScheduler {
 pub fn new(workers: usize) -> Self {
 Self { workers }
 }

 /// executetaskservice
 pub fn execute(&self, tasks: Vec<CompileTask>) -> Result<Vec<CompileOutput>, ParallelError> {
 if tasks.is_empty() {
 return Ok(vec![]);
 }

 // createtaskserviceQueue
 let queue = Arc::new(Mutex::new(TaskQueue::new(tasks)));
 
 // createresultCollector
 let results = Arc::new(Mutex::new(Vec::new()));
 
 // createworkmakeThread
 let mut handles: Vec<JoinHandle<()>> = vec![];
 
 for _ in 0..self.workers {
 let queue = Arc::clone(&queue);
 let results = Arc::clone(&results);
 
 let handle = thread::spawn(move || {
 loop {
 // Gettaskservice
 let task = {
 let mut q = queue.lock().unwrap();
 q.pop()
 };
 
 match task {
 Some(t) => {
 // executetaskservice
 let result = Self::execute_task(&t);
 
 // existresult
 let mut r = results.lock().unwrap();
 r.push(result);
 }
 None => break, // finiteupdatemanytaskservice
 }
 }
 });
 
 handles.push(handle);
 }
 
 // waitplacefiniteThreadComplete
 for handle in handles {
 handle.join().map_err(|_| ParallelError::ThreadPanic)?;
 }
 
 // receivecollectionresult
 let results = Arc::try_unwrap(results)
 .map_err(|_| ParallelError::ThreadPanic)?
 .into_inner()
 .map_err(|_| ParallelError::ThreadPanic)?;
 
 Ok(results)
 }

 /// executeformitemtaskservice
 fn execute_task(task: &CompileTask) -> CompileOutput {
 let start = Instant::now();
 
 // TODO: Call the actual compiler
 let success = true;
 
 CompileOutput {
 source: task.source.clone(),
 output: task.output.clone(),
 success,
 duration_ms: start.elapsed().as_millis() as u64,
 }
 }
}

/// taskserviceQueue
struct TaskQueue {
 tasks: VecDeque<CompileTask>,
}

impl TaskQueue {
 fn new(tasks: Vec<CompileTask>) -> Self {
 Self {
 tasks: tasks.into(),
 }
 }

 fn pop(&mut self) -> Option<CompileTask> {
 self.tasks.pop_front()
 }
}

/// banddependency taskserviceschedulingdevice
pub struct DependentTaskScheduler {
 workers: usize,
}

impl DependentTaskScheduler {
 pub fn new(workers: usize) -> Self {
 Self { workers }
 }

 /// bydependencyforwardorderexecutetaskservice
 pub fn execute_with_deps(
 &self,
 tasks: Vec<CompileTask>,
 deps: &std::collections::HashMap<PathBuf, Vec<PathBuf>>,
 ) -> Result<Vec<CompileOutput>, ParallelError> {
 // BuilddependencydiagramparallelComputeexecuteforwardorder
 let order = self.topological_sort(&tasks, deps);
 
 // byforwardorderexecute
 let mut results = vec![];
 let mut completed = std::collections::HashSet::new();
 
 for batch in order {
 // Parallelexecutesamelayerlevel taskservice
 let batch_results = self.execute_batch(&batch, &completed)?;
 completed.extend(batch.iter().map(|t| t.source.clone()));
 results.extend(batch_results);
 }
 
 Ok(results)
 }

 /// topologysort, Returnbylayerlevelsplitgroup taskservice
 fn topological_sort(
 &self,
 tasks: &[CompileTask],
 deps: &std::collections::HashMap<PathBuf, Vec<PathBuf>>,
 ) -> Vec<Vec<CompileTask>> {
 let mut in_degree: std::collections::HashMap<PathBuf, usize> = std::collections::HashMap::new();
 let mut task_map: std::collections::HashMap<PathBuf, &CompileTask> = std::collections::HashMap::new();
 
 // initialization
 for task in tasks {
 task_map.insert(task.source.clone(), task);
 in_degree.entry(task.source.clone()).or_insert(0);
 }
 
 // Computeentermeasurement
 for (source, dep_list) in deps {
 for dep in dep_list {
 if task_map.contains_key(dep) {
 *in_degree.entry(source.clone()).or_insert(0) += 1;
 }
 }
 }
 
 let mut result = vec![];
 let mut remaining: std::collections::HashSet<PathBuf> = tasks.iter()
 .map(|t| t.source.clone())
 .collect();
 
 while !remaining.is_empty() {
 // findexitentermeasurementas 0 taskservice
 let ready: Vec<PathBuf> = remaining.iter()
 .filter(|s| in_degree.get(*s).copied().unwrap_or(0) == 0)
 .cloned()
 .collect();
 
 if ready.is_empty() {
 // Existsloopdependency
 break;
 }
 
 // receivecollectionthislayer taskservice
 let batch: Vec<CompileTask> = ready.iter()
 .filter_map(|s| task_map.get(s).map(|t| (*t).clone()))
 .collect();
 
 // Updateentermeasurement
 for source in &ready {
 remaining.remove(source);
 if let Some(dep_list) = deps.get(source) {
 for dep in dep_list {
 if let Some(d) = in_degree.get_mut(dep) {
 *d = d.saturating_sub(1);
 }
 }
 }
 }
 
 result.push(batch);
 }
 
 result
 }

 /// executetaskservice
 fn execute_batch(
 &self,
 batch: &[CompileTask],
 _completed: &std::collections::HashSet<PathBuf>,
 ) -> Result<Vec<CompileOutput>, ParallelError> {
 let scheduler = TaskScheduler::new(self.workers);
 scheduler.execute(batch.to_vec())
 }
}