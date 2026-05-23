# Nuva OS Process Management Module

## Overview

The process management module provides complete process lifecycle management, including process creation, scheduling, and termination. The scheduling system supports five scheduling policies: CFS/RT/Deadline/Idle/EAS, with complete load tracking and load balancing mechanisms.

---

## Table of Contents

1. [Process Scheduling](#1-process-scheduling)
2. [Process Control Block](#2-process-control-block)
3. [Process System Calls](#3-process-system-calls)
4. [Load Balancing](#4-load-balancing)
5. [Scheduling Policies](#5-scheduling-policies)
6. [EAS Energy Aware Scheduling](#6-eas-energy-aware-scheduling)
7. [Deadline Scheduling](#7-deadline-scheduling)
8. [Load Tracking Mechanism](#8-load-tracking-mechanism)
9. [Signal Handling](#9-signal-handling)
10. [File Structure](#10-file-structure)

---

## 1. Process Scheduling

### 1.1 Scheduling Entity

```rust
pub struct SchedEntity {
    pub vruntime: AtomicU64,      // Virtual runtime
    pub runtime: AtomicU64,       // Actual runtime
    pub wait_time: AtomicU64,     // Wait time
    pub time_slice: AtomicU32,    // Time slice
    pub prio: AtomicU32,          // Dynamic priority
    pub static_prio: Priority,    // Static priority
    pub normal_prio: Priority,    // Normal priority
    pub policy: AtomicU32,        // Scheduling policy
    pub flags: AtomicU32,         // Flags
    pub cpus_allowed: AtomicU64,  // Allowed CPUs
    pub cpu: AtomicU32,           // Current CPU
    pub last_ran: AtomicU64,      // Last run time
    pub switches: AtomicU64,      // Context switch count
}
```

### 1.2 Run Queue

```rust
pub struct RunQueue {
    pub cpu: CpuId,               // CPU ID
    pub lock: AtomicU32,          // Spinlock
    pub nr_running: AtomicU32,    // Running process count
    pub rt_nr_running: AtomicU32, // Real-time process count
    pub min_vruntime: AtomicU64,  // Minimum virtual runtime
    pub curr: u64,                // Current process
    pub idle: u64,                // Idle process
    pub clock: AtomicU64,         // Scheduler clock
    pub load: AtomicU64,          // Load
    pub switches: AtomicU64,      // Switch count
}
```

### 1.3 Scheduler

```rust
pub struct Scheduler {
    pub nr_cpus: u32,                 // CPU count
    pub run_queues: [RunQueue; 8],    // Run queues
    pub nr_running: AtomicU32,        // Total running processes
    pub nr_switches: AtomicU64,       // Total switches
    pub sched_count: AtomicU64,       // Schedule count
    pub lb_interval: AtomicU64,       // Load balance interval
}
```

---

## 2. Process Control Block

### 2.1 Process Structure

```rust
pub struct ProcessControlBlock {
    pub process: Process,                     // Process structure
    pub files: FilesStruct,                   // File descriptor table
    pub parent: *mut ProcessControlBlock,     // Parent process
    pub children: *mut ProcessControlBlock,   // Child process list
    pub sibling: *mut ProcessControlBlock,    // Sibling process list
}
```

### 2.2 Process States

| State | Description |
|-------|-------------|
| `Created` | Newly created |
| `Ready` | Ready |
| `Running` | Running |
| `Blocked` | Blocked |
| `Zombie` | Zombie |
| `Terminated` | Terminated |

---

## 3. Process System Calls

### 3.1 Process Creation

#### fork - Create child process

| Item | Description |
|------|-------------|
| Function | Create a copy of the current process |
| Return | Parent returns child PID, child returns 0 |

**Implementation Steps**:
1. Get current process ID
2. Allocate new process ID
3. Allocate process control block
4. Copy parent address space (COW)
5. Copy file descriptor table
6. Copy signal handling
7. Set parent-child relationship
8. Add child process to scheduler queue

#### vfork - Create child process (shared address space)

| Item | Description |
|------|-------------|
| Function | Create child process sharing parent's address space |
| Feature | Parent blocks until child calls exec or exit |

#### clone - Create process or thread

| Item | Description |
|------|-------------|
| Function | Create process or thread based on flags |
| Parameters | flags, child_stack, ptid, ctid, newtls |

**Clone Flags**:

| Flag | Description |
|------|-------------|
| `CLONE_VM` | Share address space (thread) |
| `CLONE_FS` | Share filesystem info |
| `CLONE_FILES` | Share file descriptor table |
| `CLONE_SIGHAND` | Share signal handling |
| `CLONE_THREAD` | Same thread group |
| `CLONE_SETTLS` | Set TLS |

### 3.2 Program Execution

#### execve - Execute new program

| Item | Description |
|------|-------------|
| Function | Load and execute new program |
| Parameters | filename, argv, envp |

**Implementation Steps**:
1. Open executable file
2. Check file permissions
3. Read file header
4. Parse ELF format
5. Check interpreter (e.g., #!)
6. Release old address space
7. Create new address space
8. Load program segments (.text, .data, .bss)
9. Set up stack
10. Set arguments and environment variables
11. Set entry point
12. Switch to user space execution

### 3.3 Process Termination

#### exit - Terminate current process

| Item | Description |
|------|-------------|
| Function | Terminate current process |
| Parameter | status (exit status) |

**Implementation Steps**:
1. Set exit status
2. Close all open files
3. Release address space
4. Send SIGCHLD to parent
5. If parent is waiting, wake parent
6. Reassign children to init
7. Enter zombie state
8. Schedule other processes

### 3.4 Process Waiting

#### wait4 - Wait for child process state change

| Item | Description |
|------|-------------|
| Function | Wait for child process state change, reap zombie children |
| Parameters | pid, status, options, rusage |

**pid Parameter**:
- `-1`: Wait for any child process
- `0`: Wait for any child in same process group
- `> 0`: Wait for specific child process
- `< -1`: Wait for any child in specific process group

**options Parameter**:
- `WNOHANG`: Non-blocking
- `WUNTRACED`: Report stopped children
- `WCONTINUED`: Report continued children

### 3.5 Signal Sending

#### kill - Send signal to process

| Item | Description |
|------|-------------|
| Function | Send signal to process or process group |
| Parameters | pid, sig |

### 3.6 Process Information

| System Call | Function |
|-------------|----------|
| `getpid()` | Get current process ID |
| `getppid()` | Get parent process ID |
| `gettid()` | Get current thread ID |
| `getpgid(pid)` | Get process group ID |

### 3.7 Session and Process Group Management

| System Call | Function |
|-------------|----------|
| `setsid()` | Create new session |
| `setpgid(pid, pgid)` | Set process group |

### 3.8 Scheduling Control

#### sched_yield - Yield CPU

| Item | Description |
|------|-------------|
| Function | Voluntarily yield CPU |
| Implementation | Call scheduler's schedule function |

---

## 4. Load Balancing

### 4.1 Load Statistics

```rust
pub struct LoadStats {
    pub load: AtomicU64,          // Load
    pub nr_running: AtomicU32,    // Running process count
    pub nr_runnable: AtomicU32,   // Runnable process count
    pub nr_waiting: AtomicU32,    // Waiting process count
    pub avg_load: AtomicU64,      // Average load
}
```

### 4.2 Scheduling Domains

Scheduling domains are the basic unit of load balancing, forming a hierarchical structure:

```rust
pub struct SchedDomain {
    pub span: AtomicU64,              // CPU mask
    pub parent: *mut SchedDomain,     // Parent scheduling domain
    pub child: *mut SchedDomain,      // Child scheduling domain
    pub load: LoadStats,              // Load statistics
    pub balance_interval: AtomicU64,  // Balance interval
    pub last_balance: AtomicU64,      // Last balance time
    pub flags: AtomicU32,             // Flags
    pub level: u32,                   // Level
}
```

**Scheduling Domain Hierarchy**:

| Level | Description |
|-------|-------------|
| SMT (Hyper-threading) | Hyper-thread siblings on same physical core |
| MC (Multi-core) | Cores on same die |
| NUMA | Cores on same NUMA node |
| ALL | Entire system |

### 4.3 Scheduling Groups

Scheduling groups are groupings within a scheduling domain used for load balancing calculations:

```rust
pub struct SchedGroup {
    pub cpus: AtomicU64,              // CPU mask
    pub next: *mut SchedGroup,        // Next group (circular list)
    pub load: LoadStats,              // Group load statistics
    pub imbalance_pct: u32,           // Imbalance percentage threshold
    pub sched_domain: *mut SchedDomain, // Parent domain
}
```

- Each scheduling domain contains a set of scheduling groups (circular list)
- Load balancing compares load between groups, selecting the busiest group for migration
- `imbalance_pct` controls the load imbalance threshold that triggers migration

### 4.4 Load Balancer

```rust
pub struct LoadBalancer {
    pub balance_count: AtomicU64,     // Balance count
    pub migration_count: AtomicU64,   // Migration count
    pub fail_count: AtomicU64,        // Failure count
    pub max_migrations: AtomicU32,    // Max migrations
    pub interval: AtomicU64,          // Balance interval
}
```

### 4.5 CPU Affinity

```rust
pub struct CpuAffinity {
    pub default_mask: AtomicU64,      // Default CPU mask
}
```

---

## 5. Scheduling Policies

### 5.1 Scheduling Policy Types

```rust
pub enum SchedPolicy {
    Normal = 0,      // Normal scheduling (CFS)
    Fifo = 1,        // First-in-first-out (RT)
    RoundRobin = 2,  // Round-robin (RT)
    Batch = 3,       // Batch processing
    Idle = 4,        // Idle
    Deadline = 5,    // Deadline
}
```

### 5.2 CFS (Completely Fair Scheduler)

**Features**:
- Uses red-black tree to organize processes
- Based on virtual runtime (vruntime)
- Fair CPU time allocation

**vruntime Calculation**:
```
vruntime += delta_exec * NICE_0_LOAD / se->load
```

**Time Slice Calculation**:
```
time_slice = target_latency * (se->load / total_load)
```

- `target_latency`: Target scheduling latency (default 6ms)
- Minimum granularity: `sched_min_granularity` (default 0.75ms)

### 5.3 RT (Real-Time Scheduler)

**Features**:
- Higher priority than normal processes
- FIFO: First-in-first-out, does not yield
- RR: Round-robin, yields after time slice expiration
- Priority range: 0 (highest) to 99 (lowest)

---

## 6. EAS Energy Aware Scheduling

EAS (Energy Aware Scheduling) selects the most energy-efficient CPU for waking tasks when the system is not overutilized.

### 6.1 Energy Model

```rust
pub struct EnergyModel {
    pub domains: [Option<PerfDomain>; MAX_NR_PERF_DOMAINS],
    pub nr_domains: u32,
    pub total_capacity: AtomicU64,
    pub enabled: AtomicBool,
    pub overutilization_threshold: AtomicU32,
}
```

### 6.2 Performance Domains

```rust
pub struct PerfDomain {
    pub cpus: u64,
    pub nr_cpus: u32,
    pub states: [Option<PerfState>; 16],
    pub nr_states: u32,
    pub current_state: AtomicU32,
    pub name: [u8; 16],
}

pub struct PerfState {
    pub frequency: u32,   // Frequency (kHz)
    pub voltage: u32,     // Voltage (microvolts)
    pub power: u32,       // Power consumption (microwatts)
    pub cost: u64,        // Cost coefficient
}
```

### 6.3 EAS Scheduling Flow

1. Check if EAS is enabled and system is not overutilized
2. If overutilized (>80%), fall back to CFS load balancing
3. Calculate energy cost for each CPU
4. Consider migration cost (100mW penalty for non-local CPU)
5. Select CPU with minimum total energy

### 6.4 EAS Statistics

```rust
pub struct EasStats {
    pub eas_wakeups: AtomicU64,     // EAS wakeup count
    pub eas_migrations: AtomicU64,  // EAS migration count
    pub eas_fallbacks: AtomicU64,   // Fallback to CFS count
    pub energy_saved: AtomicU64,    // Energy saved (estimated)
}
```

---

## 7. Deadline Scheduling

The Deadline scheduler provides guaranteed CPU bandwidth and deadline support for real-time tasks.

### 7.1 Deadline Parameters

| Parameter | Description |
|-----------|-------------|
| `runtime` | Guaranteed CPU runtime within each period |
| `period` | Scheduling period |
| `deadline` | Absolute deadline |

### 7.2 Deadline Scheduling Rules

- Uses Earliest Deadline First (EDF) algorithm
- Higher priority than RT scheduler
- Bandwidth admission: Ensures total bandwidth of all Deadline tasks does not exceed 100%
  - `sum(runtime_i / period_i) <= 1`
- Rejects new tasks when bandwidth exceeded (`sched_setattr` returns EBUSY)

### 7.3 Deadline and EAS Relationship

- Deadline tasks have the highest scheduling priority
- EAS does not apply to Deadline tasks (Deadline ignores energy model)
- Deadline tasks remain guaranteed even when system is overutilized

---

## 8. Load Tracking Mechanism

### 8.1 Per-Entity Load Tracking (PELT)

PELT independently tracks load contribution for each scheduling entity:

**Load Calculation**:
```
load_sum = load_sum * y + running
load_avg = load_sum / (LOAD_AVG_MAX - 1024)
```

- `y`: Decay factor, approximately 0.98 (corresponding to 1ms decay period)
- `running`: Whether currently running (1 or 0)
- Historical load within 1024 periods (approximately 1 second) decays exponentially

### 8.2 Runtime Tracking

```
runnable_sum = runnable_sum * y + runnable
runnable_avg = runnable_sum / (LOAD_AVG_MAX - 1024)
```

- `load_avg`: Reflects task's load contribution on CPU
- `runnable_avg`: Reflects task's waiting time in run queue

### 8.3 CPU Load Update

On each scheduler tick:
1. Update current task's PELT load
2. Update run queue's CPU load
3. Update scheduling domain's load statistics
4. Trigger periodic load balancing check

---

## 9. Signal Handling

### 9.1 Signal Types

| Category | Signal | Description |
|----------|--------|-------------|
| Terminate | SIGTERM (15) | Termination request (catchable) |
| Kill | SIGKILL (9) | Forced termination (uncatchable) |
| Interrupt | SIGINT (2) | Interrupt (Ctrl+C) |
| Hangup | SIGHUP (1) | Terminal hangup |
| Segfault | SIGSEGV (11) | Invalid memory access |
| Bus Error | SIGBUS (7) | Bus error |
| Child | SIGCHLD (17) | Child process state change |
| Pipe | SIGPIPE (13) | Broken pipe |
| Real-time | SIGRTMIN-SIGRTMAX | Real-time signals (queueable) |

### 9.2 Signal Handling Flow

1. **Signal Send**: `kill(pid, sig)` or `raise(sig)`
2. **Signal Pending**: Add to target process's pending signal set
3. **Signal Check**: Check pending signals before returning to user space from kernel
4. **Signal Delivery**:
   - Default handling: Execute default action (terminate/ignore/stop/continue)
   - Custom handling: Jump to handler registered via `sigaction`
   - Ignore: Discard signal

### 9.3 Signal Mask

- `sigprocmask`: Block/unblock signals
- `sigpending`: Query pending blocked signals
- Blocked signals are not lost (pending wait), but real-time signals have queueing limits

### 9.4 Signal-Related System Calls

| System Call | Function |
|-------------|----------|
| `kill(pid, sig)` | Send signal to process |
| `tgkill(tgid, tid, sig)` | Send signal to thread |
| `raise(sig)` | Send signal to self |
| `sigaction(sig, act, oact)` | Set signal handler |
| `sigprocmask(how, set, oset)` | Set signal mask |
| `sigpending(set)` | Get pending signal set |
| `sigsuspend(mask)` | Wait for signal (atomically replace mask and suspend) |
| `pause()` | Wait for any signal |
| `alarm(seconds)` | Set timer (SIGALRM) |

---

## 10. File Structure

```
kernel/sched/
├── scheduler.rs        # Scheduler basics
├── cfs.rs              # CFS scheduler
├── rt.rs               # RT scheduler
├── eas.rs              # Energy Aware Scheduling
├── core.rs             # Core scheduling
├── load_balance.rs     # Load balancing
├── sched_domain.rs     # Scheduling domains and groups
├── rbtree.rs           # Red-black tree (CFS)
├── task.rs             # Task management
├── quant_sched.rs      # Quantum scheduler
├── ai_sched.rs         # AI scheduler

kernel/syscall/
└── process_integration.rs  # Process management system calls
```

---

**Last Updated**: May 15, 2026
**License**: Apache-2.0
