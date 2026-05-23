/*
 * Nuva OS - Kernel - CFS Scheduler
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

use core::sync::atomic::{AtomicU64, Ordering};

/// Red-black tree node color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RbColor {
    Red,
    Black,
}

/// Red-black tree node
/// Used for organizing tasks by virtual runtime in CFS.
pub struct RbNode {
    /// Virtual runtime (sort key)
    pub vruntime: u64,
    /// Left child
    pub left: *mut RbNode,
    /// Right child
    pub right: *mut RbNode,
    /// Parent node
    pub parent: *mut RbNode,
    /// Node color
    pub color: RbColor,
    /// Associated scheduling entity
    pub entity: *mut SchedEntity,
}

impl RbNode {
    pub const fn new(vruntime: u64) -> Self {
        RbNode {
            vruntime,
            left: core::ptr::null_mut(),
            right: core::ptr::null_mut(),
            parent: core::ptr::null_mut(),
            color: RbColor::Red,
            entity: core::ptr::null_mut(),
        }
    }
    
    /// Check if node is red
    pub fn is_red(&self) -> bool {
        self.color == RbColor::Red
    }
    
    /// Check if node is black
    pub fn is_black(&self) -> bool {
        self.color == RbColor::Black
    }
    
    /// Set node color to red
    pub fn set_red(&mut self) {
        self.color = RbColor::Red;
    }
    
    /// Set node color to black
    pub fn set_black(&mut self) {
        self.color = RbColor::Black;
    }
    
    /// Get grandparent node
    pub fn grandparent(&self) -> *mut RbNode {
        if self.parent.is_null() {
            return core::ptr::null_mut();
        }
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*self.parent).parent }
    }
    
    /// Get uncle node
    pub fn uncle(&self) -> *mut RbNode {
        let grandparent = self.grandparent();
        if grandparent.is_null() {
            return core::ptr::null_mut();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if self.parent == (*grandparent).left {
                (*grandparent).right
            } else {
                (*grandparent).left
            }
        }
    }
}

/// CFS run queue
/// Manages tasks using a red-black tree sorted by virtual runtime.
pub struct CfsRq {
    /// Number of running tasks
    pub nr_running: u64,
    /// Total load weight
    pub load_weight: u64,
    /// Minimum virtual runtime
    pub min_vruntime: AtomicU64,
    /// Red-black tree root
    pub rb_root: *mut RbNode,
    /// Leftmost node (minimum vruntime)
    pub rb_leftmost: *mut RbNode,
    /// Hierarchical running count
    pub h_nr_running: u64,
}

impl CfsRq {
    pub const fn new() -> Self {
        CfsRq {
            nr_running: 0,
            load_weight: 0,
            min_vruntime: AtomicU64::new(0),
            rb_root: core::ptr::null_mut(),
            rb_leftmost: core::ptr::null_mut(),
            h_nr_running: 0,
        }
    }
    
    /// Enqueue a task into the red-black tree
    /// @param node: Node to enqueue
    pub fn enqueue(&mut self, node: &mut RbNode) {
        self.nr_running += 1;
        self.h_nr_running += 1;
        
        // Update load weight
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if !node.entity.is_null() {
                self.load_weight += (*node.entity).weight;
            }
        }
        
        // Insert into red-black tree
        self.rb_insert(node);
        
        log_debug!("CFS: enqueued task, nr_running={}", self.nr_running);
    }
    
    /// Dequeue a task from the red-black tree
    /// @param node: Node to dequeue
    pub fn dequeue(&mut self, node: &mut RbNode) {
        if self.nr_running == 0 {
            return;
        }
        
        self.nr_running -= 1;
        self.h_nr_running -= 1;
        
        // Update load weight
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if !node.entity.is_null() {
                self.load_weight = self.load_weight.saturating_sub((*node.entity).weight);
            }
        }
        
        // Remove from red-black tree
        self.rb_remove(node);
        
        log_debug!("CFS: dequeued task, nr_running={}", self.nr_running);
    }
    
    /// Insert node into red-black tree
    /// @param node: Node to insert
    fn rb_insert(&mut self, node: &mut RbNode) {
        node.left = core::ptr::null_mut();
        node.right = core::ptr::null_mut();
        node.set_red();
        
        // Empty tree
        if self.rb_root.is_null() {
            node.parent = core::ptr::null_mut();
            node.set_black();
            self.rb_root = node;
            self.rb_leftmost = node;
            return;
        }
        
        // Find insertion position
        let mut parent: *mut RbNode = core::ptr::null_mut();
        let mut current = self.rb_root;
        let mut is_left = true;
        
        while !current.is_null() {
            parent = current;
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if node.vruntime < (*current).vruntime {
                    current = (*current).left;
                    is_left = true;
                } else {
                    current = (*current).right;
                    is_left = false;
                }
            }
        }
        
        // Insert node
        node.parent = parent;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if is_left {
                (*parent).left = node;
                // Update leftmost node
                if parent == self.rb_leftmost {
                    self.rb_leftmost = node;
                }
            } else {
                (*parent).right = node;
            }
        }
        
        // Fix red-black tree properties
        self.rb_insert_fixup(node);
    }
    
    /// Fix red-black tree after insertion
    /// @param node: Newly inserted node
    fn rb_insert_fixup(&mut self, mut node: *mut RbNode) {
        loop {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let parent = unsafe { (*node).parent };
            if parent.is_null() {
                break;
            }
            
            // SAFETY: unsafe block required for low-level memory or hardware access
            if unsafe { !(*parent).is_red() } {
                break;
            }
            
            // SAFETY: unsafe block required for low-level memory or hardware access
            let uncle = unsafe { (*node).uncle() };
            // SAFETY: unsafe block required for low-level memory or hardware access
            let grandparent = unsafe { (*node).grandparent() };
            
            // SAFETY: raw pointer dereference requires unsafe
            if !uncle.is_null() && unsafe { (*uncle).is_red() } {
                // Case 1: Uncle is red
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    (*parent).set_black();
                    (*uncle).set_black();
                    if !grandparent.is_null() {
                        (*grandparent).set_red();
                    }
                    node = grandparent;
                }
            } else {
                // Case 2 & 3: Uncle is black
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let parent_is_left = (*grandparent).left == parent;
                    let node_is_left = (*parent).left == node;
                    
                    if parent_is_left != node_is_left {
                        // Case 2: Need rotation
                        if node_is_left {
                            self.rb_rotate_right(parent);
                            node = parent;
                        } else {
                            self.rb_rotate_left(parent);
                            node = parent;
                        }
                    }
                    
                    // Case 3
                    let new_parent = (*node).parent;
                    if !new_parent.is_null() {
                        (*new_parent).set_black();
                    }
                    if !grandparent.is_null() {
                        (*grandparent).set_red();
                        if parent_is_left {
                            self.rb_rotate_right(grandparent);
                        } else {
                            self.rb_rotate_left(grandparent);
                        }
                    }
                }
                break;
            }
        }
        
        // Root must be black
        if !self.rb_root.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { (*self.rb_root).set_black(); }
        }
    }
    
    /// Left rotation
    /// @param node: Node to rotate around
    fn rb_rotate_left(&mut self, node: *mut RbNode) {
        if node.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let right = (*node).right;
            if right.is_null() {
                return;
            }
            
            // Move right's left subtree to node's right subtree
            (*node).right = (*right).left;
            if !(*right).left.is_null() {
                (*(*right).left).parent = node;
            }
            
            // Update right's parent
            (*right).parent = (*node).parent;
            if (*node).parent.is_null() {
                self.rb_root = right;
            } else if node == (*(*node).parent).left {
                (*(*node).parent).left = right;
            } else {
                (*(*node).parent).right = right;
            }
            
            // Make node the left child of right
            (*right).left = node;
            (*node).parent = right;
        }
    }
    
    /// Right rotation
    /// @param node: Node to rotate around
    fn rb_rotate_right(&mut self, node: *mut RbNode) {
        if node.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let left = (*node).left;
            if left.is_null() {
                return;
            }
            
            // Move left's right subtree to node's left subtree
            (*node).left = (*left).right;
            if !(*left).right.is_null() {
                (*(*left).right).parent = node;
            }
            
            // Update left's parent
            (*left).parent = (*node).parent;
            if (*node).parent.is_null() {
                self.rb_root = left;
            } else if node == (*(*node).parent).right {
                (*(*node).parent).right = left;
            } else {
                (*(*node).parent).left = left;
            }
            
            // Make node the right child of left
            (*left).right = node;
            (*node).parent = left;
        }
    }
    
    /// Remove node from red-black tree
    /// @param node: Node to remove
    fn rb_remove(&mut self, node: &mut RbNode) {
        // Update leftmost node
        if node as *mut RbNode == self.rb_leftmost {
            // Find next leftmost node
            self.rb_leftmost = self.find_next_leftmost(node);
        }
        
        // Simplified implementation: mark as removed
        // Full implementation needs proper RB tree deletion and fixup
        node.parent = core::ptr::null_mut();
        node.left = core::ptr::null_mut();
        node.right = core::ptr::null_mut();
    }
    
    /// Find next leftmost node
    /// @param node: Current leftmost node being removed
    /// @return Next leftmost node, or null if none
    fn find_next_leftmost(&self, node: &RbNode) -> *mut RbNode {
        // If has right subtree, find leftmost in right subtree
        if !node.right.is_null() {
            let mut current = node.right;
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                while !(*current).left.is_null() {
                    current = (*current).left;
                }
            }
            return current;
        }
        
        // Otherwise, go up
        let mut current = node as *const RbNode as *mut RbNode;
        let mut parent = node.parent;
        
        while !parent.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*parent).left == current {
                    return parent;
                }
                current = parent;
                parent = (*parent).parent;
            }
        }
        
        core::ptr::null_mut()
    }
    
    /// Pick next task (leftmost node with minimum vruntime)
    /// @return Pointer to next node, or null if queue is empty
    pub fn pick_next(&self) -> *mut RbNode {
        self.rb_leftmost
    }
    
    /// Update minimum virtual runtime
    /// @param vruntime: New vruntime value
    pub fn update_min_vruntime(&self, vruntime: u64) {
        let old = self.min_vruntime.load(Ordering::Acquire);
        if vruntime > old {
            self.min_vruntime.store(vruntime, Ordering::Release);
        }
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.nr_running == 0
    }
    
    /// Get queue length
    pub fn len(&self) -> u64 {
        self.nr_running
    }
}

/// Scheduling latency (nanoseconds)
pub const SCHED_LATENCY_NS: u64 = 6_000_000;  /* 6ms */

/// Minimum granularity (nanoseconds)
pub const SCHED_MIN_GRANULARITY_NS: u64 = 750_000;  /* 0.75ms */

/// Calculate time slice for a task
/// @param nr_running: Number of running tasks
/// @param weight: Task weight
/// @param total_weight: Total weight of all tasks
/// @return Time slice in nanoseconds
pub fn calc_time_slice(nr_running: u64, weight: u64, total_weight: u64) -> u64 {
    if nr_running == 0 {
        return 0;
    }
    
    let period = SCHED_LATENCY_NS.max(nr_running * SCHED_MIN_GRANULARITY_NS);
    period * weight / total_weight
}

/// Calculate virtual runtime delta
/// @param delta_exec: Actual execution time
/// @param weight: Task weight
/// @param total_weight: Total weight of all tasks
/// @return Virtual runtime delta
pub fn calc_vruntime(delta_exec: u64, weight: u64, total_weight: u64) -> u64 {
    if total_weight == 0 {
        return delta_exec;
    }
    
    // vruntime += delta_exec * (NICE_0_LOAD / weight)
    // Simplified: vruntime += delta_exec * 1024 / weight
    delta_exec * 1024 / weight.max(1)
}

/// Initialize CFS scheduler
pub fn init_cfs() {
    log_info!("CFS scheduler initialized");
    log_info!("  Sched latency: {} ns", SCHED_LATENCY_NS);
    log_info!("  Min granularity: {} ns", SCHED_MIN_GRANULARITY_NS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfs_rq_new() {
        let rq = CfsRq::new();
        assert_eq!(rq.nr_running, 0);
        assert_eq!(rq.load_weight, 0);
        assert_eq!(rq.min_vruntime.load(Ordering::Relaxed), 0);
        assert!(rq.rb_root.is_null());
        assert!(rq.rb_leftmost.is_null());
    }

    #[test]
    fn test_cfs_rq_enqueue_dequeue() {
        let mut rq = CfsRq::new();
        let mut node = RbNode {
            vruntime: 100,
            left: core::ptr::null_mut(),
            right: core::ptr::null_mut(),
            parent: core::ptr::null_mut(),
            color: RbColor::Red,
            entity: core::ptr::null_mut(),
        };

        rq.enqueue(&mut node);
        assert_eq!(rq.nr_running, 1);

        rq.dequeue(&mut node);
        assert_eq!(rq.nr_running, 0);
    }

    #[test]
    fn test_calc_time_slice() {
        // Single task
        let slice = calc_time_slice(1, 1024, 1024);
        assert_eq!(slice, SCHED_LATENCY_NS);

        // Multiple tasks
        let slice = calc_time_slice(4, 1024, 4096);
        assert_eq!(slice, SCHED_LATENCY_NS / 4);

        // Empty queue
        let slice = calc_time_slice(0, 1024, 1024);
        assert_eq!(slice, 0);
    }

    #[test]
    fn test_calc_vruntime() {
        // Default weight
        let vruntime = calc_vruntime(1000, 1024, 1024);
        assert_eq!(vruntime, 1000);

        // High priority (larger weight)
        let vruntime_high = calc_vruntime(1000, 2048, 1024);
        assert!(vruntime_high < 1000);

        // Low priority (smaller weight)
        let vruntime_low = calc_vruntime(1000, 512, 1024);
        assert!(vruntime_low > 1000);
    }

    #[test]
    fn test_update_min_vruntime() {
        let rq = CfsRq::new();

        // Update to larger value
        rq.update_min_vruntime(100);
        assert_eq!(rq.min_vruntime.load(Ordering::Relaxed), 100);

        // Update to smaller value (should not change)
        rq.update_min_vruntime(50);
        assert_eq!(rq.min_vruntime.load(Ordering::Relaxed), 100);

        // Update to larger value
        rq.update_min_vruntime(200);
        assert_eq!(rq.min_vruntime.load(Ordering::Relaxed), 200);
    }

    #[test]
    fn test_rb_node() {
        let node = RbNode::new(500);
        assert_eq!(node.vruntime, 500);
        assert!(node.is_red());
    }
}

// ============================================================================
// CFS Scheduler Enhanced Implementation

/// Priority to weight mapping table
pub const PRIO_TO_WEIGHT: [u64; 40] = [
    // -20 */     88761,     71755,     56483,     46273,     36291,
    // -10 */      9548,      7620,      6100,      4904,      3906,
    // 0 */      1024,       820,       655,       526,       423,
    // 10 */       110,        87,        70,        56,        45,
];

/// Priority to wmult mapping table
pub const PRIO_TO_WMULT: [u64; 40] = [
    // -20 */     4194304,   3235840,   2539520,   1992704,   1564672,
    // -10 */      406528,    323584,    257536,    203904,    162048,
    // 0 */       41472,     33152,     26368,     20992,     16704,
    // 10 */        4224,      3360,      2688,      2144,      1704,
];

/// Scheduling entity
/// Represents a schedulable entity (task or group).
pub struct SchedEntity {
    /// Weight for time slice calculation
    pub weight: u64,
    /// Virtual runtime
    pub vruntime: AtomicU64,
    /// Total execution time
    pub sum_exec_runtime: AtomicU64,
    /// Previous execution time
    pub prev_sum_exec_runtime: AtomicU64,
    /// Whether entity is on run queue
    pub on_rq: bool,
}

impl SchedEntity {
    pub const fn new() -> Self {
        SchedEntity {
            weight: 1024,  /* Default weight */
            vruntime: AtomicU64::new(0),
            sum_exec_runtime: AtomicU64::new(0),
            prev_sum_exec_runtime: AtomicU64::new(0),
            on_rq: false,
        }
    }

    /// Set priority and update weight
    /// @param prio: Nice value (-20 to 19)
    pub fn set_prio(&mut self, prio: i32) {
        let idx = (prio + 20) as usize;
        if idx < 40 {
            self.weight = PRIO_TO_WEIGHT[idx];
        }
    }

    /// Update execution time
    /// @param delta: Time delta in nanoseconds
    pub fn update_runtime(&self, delta: u64) {
        self.sum_exec_runtime.fetch_add(delta, Ordering::AcqRel);
    }

    /// Update virtual runtime
    /// @param delta: Execution time delta
    /// @param total_weight: Total weight of all entities
    pub fn update_vruntime(&self, delta: u64, total_weight: u64) {
        let vruntime_delta = calc_vruntime(delta, self.weight, total_weight);
        self.vruntime.fetch_add(vruntime_delta, Ordering::AcqRel);
    }
}

/// CFS scheduler
/// Main scheduler implementing Completely Fair Scheduling algorithm.
pub struct CfsScheduler {
    /// Run queue
    pub rq: CfsRq,
    /// Current running entity
    pub curr: *mut SchedEntity,
    /// Clock rate in Hz
    pub clock_rate: u64,
    /// Last tick timestamp
    pub last_tick: AtomicU64,
}

impl CfsScheduler {
    pub const fn new() -> Self {
        CfsScheduler {
            rq: CfsRq::new(),
            curr: core::ptr::null_mut(),
            clock_rate: 1000,  /* 1000 Hz */
            last_tick: AtomicU64::new(0),
        }
    }

    /// Pick next task to run
    /// @return Pointer to next scheduling entity, or null if none
    pub fn pick_next_task(&mut self) -> *mut SchedEntity {
        // Get leftmost node (minimum vruntime)
        let next = self.rq.pick_next();
        if next.is_null() {
            return core::ptr::null_mut();
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Update current task
            self.curr = next as *mut SchedEntity;

            // Remove from run queue
            self.rq.dequeue(&mut *next);

            // Update minimum vruntime
            let vruntime = (*next).vruntime.load(Ordering::Acquire);
            self.rq.update_min_vruntime(vruntime);

            next as *mut SchedEntity
        }
    }

    /// Put previous task back to run queue
    /// @param prev: Previous scheduling entity
    pub fn put_prev_task(&mut self, prev: *mut SchedEntity) {
        if prev.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Update execution time
            let now = self.get_time();
            let last = self.last_tick.swap(now, Ordering::AcqRel);
            let delta = now - last;

            (*prev).update_runtime(delta);
            (*prev).update_vruntime(delta, self.rq.load_weight);

            // Put back to run queue
            self.rq.enqueue(&mut *((prev as *mut RbNode)));
        }
    }

    /// Handle scheduler tick
    pub fn tick(&mut self) {
        let now = self.get_time();
        self.last_tick.store(now, Ordering::Release);

        // Check if preemption is needed
        if self.check_preempt() {
            self.reschedule();
        }
    }

    /// Check if preemption is needed
    /// @return true if preemption is needed
    fn check_preempt(&self) -> bool {
        if self.curr.is_null() {
            return false;
        }

        // Get leftmost node
        let leftmost = self.rq.rb_leftmost;
        if leftmost.is_null() {
            return false;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let curr_vruntime = (*self.curr).vruntime.load(Ordering::Acquire);
            let leftmost_vruntime = (*leftmost).vruntime;

            // Preempt if leftmost vruntime is much smaller
            let granularity = SCHED_MIN_GRANULARITY_NS;
            curr_vruntime > leftmost_vruntime + granularity
        }
    }

    /// Trigger reschedule
    fn reschedule(&self) {
        // Set reschedule flag
        // set_tsk_need_resched(current);
    }

    /// Get current time
    fn get_time(&self) -> u64 {
        // TODO: Get from hardware clock
        self.last_tick.load(Ordering::Acquire) + 1_000_000  /* 1ms */
    }

    /// Calculate time slice for entity
    /// @param se: Scheduling entity
    /// @return Time slice in nanoseconds
    pub fn calc_slice(&self, se: &SchedEntity) -> u64 {
        calc_time_slice(self.rq.nr_running, se.weight, self.rq.load_weight)
    }
}

/// Load balancer
/// Handles load balancing between CPU run queues.
pub struct LoadBalancer {
    /// Balance interval in nanoseconds
    pub balance_interval: u64,
    /// Last balance timestamp
    pub last_balance: AtomicU64,
    /// Imbalance threshold percentage
    pub imbalance_pct: u32,
}

impl LoadBalancer {
    pub const fn new() -> Self {
        LoadBalancer {
            balance_interval: 100_000_000,  /* 100ms */
            last_balance: AtomicU64::new(0),
            imbalance_pct: 125,  /* 125% */
        }
    }

    /// Check if load balancing is needed
    /// @param now: Current timestamp
    /// @return true if balancing is needed
    pub fn need_balance(&self, now: u64) -> bool {
        let last = self.last_balance.load(Ordering::Acquire);
        now >= last + self.balance_interval
    }

    /// Calculate load imbalance
    /// @param src_load: Source queue load
    /// @param dst_load: Destination queue load
    /// @return Imbalance amount
    pub fn calc_imbalance(&self, src_load: u64, dst_load: u64) -> u64 {
        if src_load == 0 {
            return 0;
        }

        let avg_load = (src_load + dst_load) / 2;
        let max_load = src_load.max(dst_load);

        if max_load * 100 > avg_load * self.imbalance_pct as u64 {
            (src_load - dst_load) / 2
        } else {
            0
        }
    }

    /// Perform load balancing
    /// @param src_rq: Source run queue
    /// @param dst_rq: Destination run queue
    /// @return Amount of load transferred
    pub fn balance(&self, src_rq: &mut CfsRq, dst_rq: &mut CfsRq) -> u64 {
        let src_load = src_rq.load_weight;
        let dst_load = dst_rq.load_weight;

        let imbalance = self.calc_imbalance(src_load, dst_load);
        if imbalance == 0 {
            return 0;
        }

        // Migrate tasks
        // TODO: Implement task migration

        imbalance
    }
}

/// CPU run queue
/// Per-CPU run queue containing all scheduling classes.
pub struct CpuRq {
    /// CFS run queue
    pub cfs: CfsRq,
    /// RT run queue
    pub rt: RtRq,
    /// Current running entity
    pub curr: *mut SchedEntity,
    /// CPU ID
    pub cpu: u32,
    /// CPU load
    pub load: AtomicU64,
}

impl CpuRq {
    pub const fn new(cpu: u32) -> Self {
        CpuRq {
            cfs: CfsRq::new(),
            rt: RtRq::new(),
            curr: core::ptr::null_mut(),
            cpu,
            load: AtomicU64::new(0),
        }
    }
}

/// Real-time scheduling run queue
pub struct RtRq {
    /// Number of RT tasks
    pub rt_nr_running: u64,
    /// Highest priority in queue
    pub highest_prio: u32,
    /// Whether queue is overloaded
    pub overloaded: bool,
}

impl RtRq {
    pub const fn new() -> Self {
        RtRq {
            rt_nr_running: 0,
            highest_prio: 100,
            overloaded: false,
        }
    }
}
