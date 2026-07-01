/*
 * Nuva OS - Kernel - Wait4 Implementation
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
use crate::{pr_debug, pr_info};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Process ID type
pub type Pid = u32;

/// Wait options
pub mod wait_options {
    /// Return immediately if no child has exited
    pub const WNOHANG: i32 = 0x00000001;
    /// Return if child has stopped
    pub const WUNTRACED: i32 = 0x00000002;
    /// Return if child has continued
    pub const WCONTINUED: i32 = 0x00000008;
    /// Wait for any child process
    pub const WANY: i32 = -1;
}

/// Wait status codes
pub mod wait_status {
    /// Normal exit
    pub const W_EXITCODE: u32 = 0;
    /// Killed by signal
    pub const W_SIGNALED: u32 = 1;
    /// Stopped by signal
    pub const W_STOPPED: u32 = 2;
    /// Continued
    pub const W_CONTINUED: u32 = 3;
}

/// Wait queue entry
/// Represents a task waiting on a wait queue.
#[repr(C)]
pub struct WaitQueueEntry {
    /// Next entry in queue
    pub next: *mut WaitQueueEntry,
    /// Previous entry in queue
    pub prev: *mut WaitQueueEntry,
    /// Task waiting
    pub task: u64,
    /// Wait flags
    pub flags: u32,
}

impl WaitQueueEntry {
    pub const fn new() -> Self {
        WaitQueueEntry {
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
            task: 0,
            flags: 0,
        }
    }
}

/// Wait queue
/// Queue of tasks waiting for an event.
pub struct WaitQueue {
    /// Queue head
    pub head: *mut WaitQueueEntry,
    /// Number of waiters
    pub count: AtomicU32,
}

impl WaitQueue {
    pub const fn new() -> Self {
        WaitQueue {
            head: core::ptr::null_mut(),
            count: AtomicU32::new(0),
        }
    }
    
    /// Initialize wait queue
    pub fn init(&mut self) {
        self.head = core::ptr::null_mut();
        self.count.store(0, Ordering::Release);
    }
    
    /// Add entry to wait queue
    pub fn add(&mut self, entry: *mut WaitQueueEntry) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if self.head.is_null() {
                // First entry
                (*entry).next = entry;
                (*entry).prev = entry;
                self.head = entry;
            } else {
                // Add to tail
                let tail = (*self.head).prev;
                (*entry).next = self.head;
                (*entry).prev = tail;
                (*tail).next = entry;
                (*self.head).prev = entry;
            }
        }
        self.count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Remove entry from wait queue
    pub fn remove(&mut self, entry: *mut WaitQueueEntry) {
        if entry.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let next = (*entry).next;
            let prev = (*entry).prev;
            
            if next == entry {
                // Only one entry
                self.head = core::ptr::null_mut();
            } else {
                (*prev).next = next;
                (*next).prev = prev;
                if self.head == entry {
                    self.head = next;
                }
            }
            
            (*entry).next = core::ptr::null_mut();
            (*entry).prev = core::ptr::null_mut();
        }
        
        self.count.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Wake up one waiter
    pub fn wake_up(&mut self) {
        if self.head.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let entry = self.head;
            self.remove(entry);
            
            // Mark task as runnable
            self.wake_task((*entry).task);
        }
    }
    
    /// Wake up all waiters
    pub fn wake_up_all(&mut self) {
        while !self.head.is_null() {
            self.wake_up();
        }
    }
    
    /// Wake up a task
    fn wake_task(&self, task: u64) {
        // Set the task state to TASK_RUNNING and add it to
        // the scheduler's run queue so it will be scheduled.
        // In a full implementation:
        // let t = task as *mut TaskStruct;
        // if (*t).state != TASK_RUNNING {
        // (*t).state = TASK_RUNNING;
        // enqueue_task(rq, t);
        // resched_cpu(cpu_of(rq));
        // }
        let _ = task;
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }
    
    /// Get number of waiters
    pub fn len(&self) -> u32 {
        self.count.load(Ordering::Acquire)
    }
}

/// Child wait info
/// Information about a child process for wait.
pub struct ChildWaitInfo {
    /// Child PID
    pub pid: Pid,
    /// Exit status
    pub exit_status: i32,
    /// Wait status type
    pub wait_status: u32,
    /// Resource usage
    pub utime: u64,
    pub stime: u64,
}

/// Wait4 handler
/// Implements wait4, waitpid, and related system calls.
pub struct Wait4Handler {
    /// Total wait calls
    pub wait_count: AtomicU64,
    /// Successful waits
    pub wait_success: AtomicU64,
    /// Failed waits
    pub wait_failures: AtomicU64,
    /// No-hang returns
    pub wait_nohang: AtomicU64,
    /// Children reaped
    pub children_reaped: AtomicU64,
}

impl Wait4Handler {
    pub const fn new() -> Self {
        Wait4Handler {
            wait_count: AtomicU64::new(0),
            wait_success: AtomicU64::new(0),
            wait_failures: AtomicU64::new(0),
            wait_nohang: AtomicU64::new(0),
            children_reaped: AtomicU64::new(0),
        }
    }
    
    /// Wait for child process
    /// @param pid: PID to wait for (-1 for any child)
    /// @param status: Pointer to store exit status
    /// @param options: Wait options (WNOHANG, etc.)
    /// @param rusage: Pointer to store resource usage (can be null)
    /// @return Child PID, 0 if WNOHANG and no child, or -errno on error
    pub fn do_wait4(
        &self,
        pid: i32,
        status: *mut i32,
        options: i32,
        rusage: *mut u8,
    ) -> i64 {
        self.wait_count.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("do_wait4: pid={}, options={:#x}", pid, options);
        
        // Step 1: Validate parameters
        if !self.validate_params(pid, options) {
            self.wait_failures.fetch_add(1, Ordering::AcqRel);
            return Errno::Einval.to_syscall_return();  /* EINVAL */
        }
        
        // Step 2: Check if we have any children
        if !self.has_children() {
            self.wait_failures.fetch_add(1, Ordering::AcqRel);
            return Errno::Echild.to_syscall_return();  /* ECHILD */
        }
        
        // Step 3: Look for eligible child
        loop {
            // Find child that has changed state
            let child_info = self.find_child(pid, options);
            
            match child_info {
                Some(info) => {
                    // Found a child
                    self.wait_success.fetch_add(1, Ordering::AcqRel);
                    
                    // Store status
                    if !status.is_null() {
                        // SAFETY: unsafe block required for low-level memory or hardware access
                        unsafe {
                            *status = self.encode_status(&info);
                        }
                    }
                    
                    // Store resource usage
                    if !rusage.is_null() {
                        self.store_rusage(rusage, &info);
                    }
                    
                    // Reap zombie if exiting
                    if info.wait_status == wait_status::W_EXITCODE 
                        || info.wait_status == wait_status::W_SIGNALED 
                    {
                        self.reap_child(info.pid);
                        self.children_reaped.fetch_add(1, Ordering::AcqRel);
                    }
                    
                    return info.pid as i64;
                }
                None => {
                    // No eligible child found
                    if (options & wait_options::WNOHANG) != 0 {
                        // Non-blocking, return immediately
                        self.wait_nohang.fetch_add(1, Ordering::AcqRel);
                        return 0;
                    }
                    
                    // Blocking wait - sleep and retry
                    let result = self.sleep_wait();
                    if result.is_err() {
                        // Interrupted by signal
                        self.wait_failures.fetch_add(1, Ordering::AcqRel);
                        return result.err().unwrap() as i64;
                    }
                    
                    // Check again
                }
            }
        }
    }
    
    /// Validate wait parameters
    fn validate_params(&self, pid: i32, options: i32) -> bool {
        // Check for valid options
        let valid_options = wait_options::WNOHANG 
            | wait_options::WUNTRACED 
            | wait_options::WCONTINUED;
        
        if (options & !valid_options) != 0 {
            return false;
        }
        
        // pid can be -1, 0, or > 0
        // -1: wait for any child
        // 0: wait for any child in same process group
        // > 0: wait for specific child
        // < -1: wait for any child in specified process group
        
        true
    }
    
    /// Check if current process has children
    fn has_children(&self) -> bool {
        // Check if the current process has any child processes.
        // In a full implementation:
        // let current = current_task();
        // !list_empty(&current->children)
        // We walk the task's children list. If it's empty,
        // wait4 should return ECHILD.
        true
    }
    
    /// Find eligible child
    fn find_child(&self, pid: i32, options: i32) -> Option<ChildWaitInfo> {
        // Search the current process's children list for a child
        // that matches the pid criteria and has changed state.
        // In a full implementation:
        // let current = current_task();
        // for child in &current->children {
        // // Check if child matches pid criteria
        // if !self.child_matches(child, pid) {
        // continue;
        // }
        // // Check if child has changed state
        // if self.child_changed(child, options) {
        // return Some(self.get_child_info(child));
        // }
        // }
        // return None;
        if pid > 0 {
            // Specific child: look up by PID and check if it
            // is our child and has changed state.
            // In a full implementation:
            // let child = find_task_by_pid(pid as u32);
            // if child.is_null() || (*child).ppid != (*current).pid {
            // return None;  // Not our child
            // }
            // if self.child_changed(child, options) {
            // return Some(self.get_child_info(child));
            // }
            Some(ChildWaitInfo {
                pid: pid as u32,
                exit_status: 0,
                wait_status: wait_status::W_EXITCODE,
                utime: 0,
                stime: 0,
            })
        } else {
            // pid == -1: any child
            // pid == 0: any child in same process group
            // pid < -1: any child in specified process group
            // Walk children list and find first eligible child.
            None
        }
    }
    
    /// Check if child matches pid criteria
    fn child_matches(&self, _child: u64, pid: i32) -> bool {
        // Check if a child process matches the pid criteria:
        // pid == -1: any child
        // pid == 0: any child in same process group as caller
        // pid > 0: specific child with this PID
        // pid < -1: any child in process group abs(pid)
        // In a full implementation:
        // if pid == -1 {
        // return true;  // Any child
        // }
        // if pid == 0 {
        // return child.pgid == current.pgid;  // Same process group
        // }
        // if pid > 0 {
        // return child.pid == pid as u32;  // Specific child
        // }
        // // pid < -1: specific process group
        // return child.pgid == (-pid) as u32;
        let _ = pid;
        true
    }
    
    /// Check if child has changed state
    fn child_changed(&self, _child: u64, options: i32) -> bool {
        // Check if a child process has changed state in a way
        // that is visible to the waiting parent.
        // A child is eligible if:
        // 1. It has exited (TASK_ZOMBIE or TASK_DEAD)
        // 2. It has stopped AND WUNTRACED is set
        // 3. It has continued AND WCONTINUED is set
        // In a full implementation:
        // // Child has exited
        // if child.state == TASK_ZOMBIE || child.state == TASK_DEAD {
        // return true;
        // }
        // // Child has stopped (if WUNTRACED)
        // if (options & WUNTRACED) != 0
        // && child.state == TASK_STOPPED
        // && !child.stop_notified
        // {
        // return true;
        // }
        // // Child has continued (if WCONTINUED)
        // if (options & WCONTINUED) != 0
        // && child.continued
        // && !child.cont_notified
        // {
        // return true;
        // }
        // return false;
        let _ = options;
        true
    }
    
    /// Get child wait info
    fn get_child_info(&self, _child: u64) -> ChildWaitInfo {
        // Extract wait information from a child task structure.
        // In a full implementation:
        // ChildWaitInfo {
        // pid: child.pid,
        // exit_status: child.exit_status,
        // wait_status: if child.state == TASK_ZOMBIE {
        // if child.exit_signal != 0 {
        // wait_status::W_SIGNALED
        // } else {
        // wait_status::W_EXITCODE
        // }
        // } else if child.state == TASK_STOPPED {
        // wait_status::W_STOPPED
        // } else {
        // wait_status::W_CONTINUED
        // },
        // utime: child.utime,
        // stime: child.stime,
        // }
        ChildWaitInfo {
            pid: 0,
            exit_status: 0,
            wait_status: wait_status::W_EXITCODE,
            utime: 0,
            stime: 0,
        }
    }
    
    /// Encode wait status
    fn encode_status(&self, info: &ChildWaitInfo) -> i32 {
        match info.wait_status {
            wait_status::W_EXITCODE => {
                // Normal exit: status = exit_code << 8
                (info.exit_status & 0xFF) << 8
            }
            wait_status::W_SIGNALED => {
                // Killed by signal: status = signal
                info.exit_status & 0x7F
            }
            wait_status::W_STOPPED => {
                // Stopped: status = 0x7F | (signal << 8)
                0x7F | ((info.exit_status & 0xFF) << 8)
            }
            wait_status::W_CONTINUED => {
                // Continued: status = 0xFFFF
                0xFFFF
            }
            _ => 0,
        }
    }
    
    /// Store resource usage
    fn store_rusage(&self, rusage: *mut u8, info: &ChildWaitInfo) {
        // TODO: Fill rusage structure
        // struct rusage {
        // struct timeval ru_utime;  // User time
        // struct timeval ru_stime;  // System time
        // long ru_maxrss;           // Max resident set size
        // long ru_ixrss;            // Integral shared memory size
        // long ru_idrss;            // Integral unshared data size
        // long ru_isrss;            // Integral unshared stack size
        // long ru_minflt;           // Minor page faults
        // long ru_majflt;           // Major page faults
        // long ru_nswap;            // Swaps
        // long ru_inblock;          // Block input operations
        // long ru_oublock;          // Block output operations
        // long ru_msgsnd;           // Messages sent
        // long ru_msgrcv;           // Messages received
        // long ru_nsignals;         // Signals received
        // long ru_nvcsw;            // Voluntary context switches
        // long ru_nivcsw;           // Involuntary context switches
        // }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Store user time (offset 0, 16 bytes for timeval)
            let utime_ptr = rusage as *mut u64;
            *utime_ptr = info.utime;
            
            // Store system time (offset 16)
            let stime_ptr = rusage.add(16) as *mut u64;
            *stime_ptr = info.stime;
        }
    }
    
    /// Sleep until child changes state
    fn sleep_wait(&self) -> Result<(), i32> {
        // Add the current task to the child wait queue and sleep
        // until a child changes state (exits, stops, or continues).
        // In a full implementation:
        // let current = current_task();
        // let wait = &current->wait_chldexit;
        // define_wait(wait_entry);
        // add_wait_queue(wait, &wait_entry);
        // loop {
        // set_current_state(TASK_INTERRUPTIBLE);
        // if child_changed() {
        // break;
        // }
        // schedule();
        // if signal_pending(current) {
        // remove_wait_queue(wait, &wait_entry);
        // set_current_state(TASK_RUNNING);
        // return Err(-4);  // EINTR
        // }
        // }
        // set_current_state(TASK_RUNNING);
        // remove_wait_queue(wait, &wait_entry);
        // The TASK_INTERRUPTIBLE state allows the sleep to be
        // interrupted by signals, which is required by POSIX.
        Ok(())
    }
    
    /// Reap zombie child
    fn reap_child(&self, pid: Pid) {
        // Release all resources held by a zombie child process.
        // This is called after the parent has collected the child's
        // exit status. The zombie's resources are freed and the
        // task structure is removed from the task list.
        // In a full implementation:
        // let child = find_task_by_pid(pid);
        // if child.is_null() {
        // log_warn!("reap_child: pid {} not found", pid);
        // return;
        // }
        // // Release memory (mm_struct and page tables)
        // mmput(child.mm);
        // // Release file descriptors
        // put_files_struct(child.files);
        // // Release signal handlers
        // put_sighand(child.sighand);
        // // Release file system info
        // put_fs_struct(child.fs);
        // // Release kernel stack
        // free_kstack(child.kstack);
        // // Mark task as dead
        // child.state = TASK_DEAD;
        // // Remove from PID hash table
        // detach_pid(child, PIDTYPE_PID);
        // // Remove from parent's children list
        // list_del(&child->sibling);
        // // Remove from task list
        // list_del(&child->tasks);
        // // Reparent children to init
        // reparent_children(child, init_task);
        // // Free task structure
        // free_task(child);
        log_debug!("reap_child: pid={}", pid);
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.wait_count.load(Ordering::Acquire),
            self.wait_success.load(Ordering::Acquire),
            self.wait_failures.load(Ordering::Acquire),
            self.wait_nohang.load(Ordering::Acquire),
            self.children_reaped.load(Ordering::Acquire),
        )
    }
}

/// Global wait4 handler
static WAIT4_HANDLER: crate::sync_oncelock::OnceLock<Wait4Handler> = crate::sync_oncelock::OnceLock::new();

/// Get wait4 handler
pub fn wait4_handler() -> &'static Wait4Handler {
    WAIT4_HANDLER.get_or_init(Wait4Handler::new)
}

/// Initialize wait4 handler
pub fn init_wait4() {
    log_info!("Wait4 handler initialized");
}

/// Wait4 system call
pub fn sys_wait4(pid: i32, status: *mut i32, options: i32, rusage: *mut u8) -> i64 {
    get_wait4_handler().do_wait4(pid, status, options, rusage)
}

/// Waitpid system call
pub fn sys_waitpid(pid: i32, status: *mut i32, options: i32) -> i64 {
    sys_wait4(pid, status, options, core::ptr::null_mut())
}

/// Wait system call
pub fn sys_wait(status: *mut i32) -> i64 {
    sys_wait4(-1, status, 0, core::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wait_options() {
        assert_eq!(wait_options::WNOHANG, 1);
        assert_eq!(wait_options::WUNTRACED, 2);
        assert_eq!(wait_options::WCONTINUED, 8);
    }
    
    #[test]
    fn test_wait_status() {
        assert_eq!(wait_status::W_EXITCODE, 0);
        assert_eq!(wait_status::W_SIGNALED, 1);
        assert_eq!(wait_status::W_STOPPED, 2);
        assert_eq!(wait_status::W_CONTINUED, 3);
    }
    
    #[test]
    fn test_wait_queue_entry_new() {
        let entry = WaitQueueEntry::new();
        assert!(entry.next.is_null());
        assert!(entry.prev.is_null());
    }
    
    #[test]
    fn test_wait_queue_new() {
        let wq = WaitQueue::new();
        assert!(wq.is_empty());
        assert_eq!(wq.len(), 0);
    }
    
    #[test]
    fn test_wait4_handler_new() {
        let handler = Wait4Handler::new();
        let (waits, success, failures, nohang, reaped) = handler.get_stats();
        assert_eq!(waits, 0);
        assert_eq!(success, 0);
        assert_eq!(failures, 0);
        assert_eq!(nohang, 0);
        assert_eq!(reaped, 0);
    }
    
    #[test]
    fn test_encode_status_exit() {
        let handler = Wait4Handler::new();
        let info = ChildWaitInfo {
            pid: 123,
            exit_status: 42,
            wait_status: wait_status::W_EXITCODE,
            utime: 0,
            stime: 0,
        };
        
        let status = handler.encode_status(&info);
        assert_eq!(status, 42 << 8);  /* Exit code shifted left by 8 */
    }
    
    #[test]
    fn test_encode_status_signaled() {
        let handler = Wait4Handler::new();
        let info = ChildWaitInfo {
            pid: 123,
            exit_status: 9,  /* SIGKILL */
            wait_status: wait_status::W_SIGNALED,
            utime: 0,
            stime: 0,
        };
        
        let status = handler.encode_status(&info);
        assert_eq!(status, 9);  /* Signal number */
    }
    
    #[test]
    fn test_encode_status_stopped() {
        let handler = Wait4Handler::new();
        let info = ChildWaitInfo {
            pid: 123,
            exit_status: 19,  /* SIGSTOP */
            wait_status: wait_status::W_STOPPED,
            utime: 0,
            stime: 0,
        };
        
        let status = handler.encode_status(&info);
        assert_eq!(status, 0x7F | (19 << 8));
    }
    
    #[test]
    fn test_encode_status_continued() {
        let handler = Wait4Handler::new();
        let info = ChildWaitInfo {
            pid: 123,
            exit_status: 0,
            wait_status: wait_status::W_CONTINUED,
            utime: 0,
            stime: 0,
        };
        
        let status = handler.encode_status(&info);
        assert_eq!(status, 0xFFFF);
    }
}
