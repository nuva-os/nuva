use crate::{pr_info, pr_warn};
/*
 * Nuva OS - Kernel - Security Defense Framework
 * 
 * System-level security defense including virus scanning,
 * threat detection, and attack interception.
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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

/// Threat level
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatLevel {
    /// Safe - no threat detected
    Safe = 0,
    /// Low - minor suspicious activity
    Low = 1,
    /// Medium - potential threat
    Medium = 2,
    /// High - confirmed threat
    High = 3,
    /// Critical - immediate danger
    Critical = 4,
}

impl Default for ThreatLevel {
    fn default() -> Self {
        Self::Safe
    }
}

/// Threat category
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatCategory {
    /// Virus/Malware
    Malware = 0,
    /// Ransomware
    Ransomware = 1,
    /// Trojan
    Trojan = 2,
    /// Worm
    Worm = 3,
    /// Spyware
    Spyware = 4,
    /// Rootkit
    Rootkit = 5,
    /// Exploit
    Exploit = 6,
    /// DoS Attack
    DoSAttack = 7,
    /// Intrusion Attempt
    Intrusion = 8,
    /// Suspicious Behavior
    Suspicious = 9,
    /// Unknown
    Unknown = 255,
}

/// Threat action
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatAction {
    /// Allow - no action taken
    Allow = 0,
    /// Warn - log warning
    Warn = 1,
    /// Quarantine - isolate threat
    Quarantine = 2,
    /// Block - block the action
    Block = 3,
    /// Delete - remove threat
    Delete = 4,
    /// Kill - terminate process
    Kill = 5,
}

/// Threat information
#[repr(C)]
pub struct ThreatInfo {
    /// Threat ID
    pub id: u64,
    /// Threat name
    pub name: [u8; 64],
    /// Category
    pub category: ThreatCategory,
    /// Level
    pub level: ThreatLevel,
    /// Description
    pub description: [u8; 256],
    /// Signature hash
    pub signature: [u8; 32],
    /// First seen timestamp
    pub first_seen: u64,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Detection count
    pub detection_count: AtomicU32,
    /// Is active
    pub is_active: AtomicBool,
}

impl Clone for ThreatInfo {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            level: self.level.clone(),
            description: self.description.clone(),
            signature: self.signature.clone(),
            first_seen: self.first_seen.clone(),
            last_seen: self.last_seen.clone(),
            detection_count: AtomicU32::new(self.detection_count.load(core::sync::atomic::Ordering::Relaxed)),
            is_active: AtomicBool::new(self.is_active.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl ThreatInfo {
    pub fn new(id: u64, name: &str, category: ThreatCategory, level: ThreatLevel) -> Self {
        let mut name_buf = [0u8; 64];
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(63);
        name_buf[..len].copy_from_slice(&name_bytes[..len]);
        
        ThreatInfo {
            id,
            name: name_buf,
            category,
            level,
            description: [0; 256],
            signature: [0; 32],
            first_seen: 0,
            last_seen: 0,
            detection_count: AtomicU32::new(0),
            is_active: AtomicBool::new(true),
        }
    }
}

/// Scan result
#[repr(C)]
pub struct ScanResult {
    /// Scan ID
    pub scan_id: u64,
    /// Target path
    pub target: [u8; 256],
    /// Total files scanned
    pub files_scanned: u64,
    /// Total bytes scanned
    pub bytes_scanned: u64,
    /// Threats found
    pub threats_found: u32,
    /// Threats list
    pub threats: [Option<ThreatInfo>; 16],
    /// Scan duration (ms)
    pub duration_ms: u64,
    /// Scan status
    pub status: ScanStatus,
}

/// Scan status
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanStatus {
    /// Not started
    Pending = 0,
    /// In progress
    InProgress = 1,
    /// Completed
    Completed = 2,
    /// Cancelled
    Cancelled = 3,
    /// Failed
    Failed = 4,
}

/// Attack type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    /// Buffer overflow
    BufferOverflow = 0,
    /// SQL injection
    SqlInjection = 1,
    /// XSS attack
    XssAttack = 2,
    /// CSRF attack
    CsrfAttack = 3,
    /// Path traversal
    PathTraversal = 4,
    /// Code injection
    CodeInjection = 5,
    /// DDoS attack
    DdosAttack = 6,
    /// Brute force
    BruteForce = 7,
    /// Port scan
    PortScan = 8,
    /// Privilege escalation
    PrivilegeEscalation = 9,
    /// Unknown attack
    Unknown = 255,
}

/// Attack event
#[repr(C)]
pub struct AttackEvent {
    /// Event ID
    pub id: u64,
    /// Attack type
    pub attack_type: AttackType,
    /// Source IP
    pub source_ip: [u8; 16],
    /// Source port
    pub source_port: u16,
    /// Target process
    pub target_pid: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Threat level
    pub level: ThreatLevel,
    /// Action taken
    pub action: ThreatAction,
    /// Details
    pub details: [u8; 512],
    /// Blocked
    pub blocked: AtomicBool,
}

/// Defense statistics
#[repr(C)]
pub struct DefenseStats {
    /// Total scans performed
    pub total_scans: AtomicU64,
    /// Total threats detected
    pub threats_detected: AtomicU64,
    /// Total threats blocked
    pub threats_blocked: AtomicU64,
    /// Total attacks intercepted
    pub attacks_intercepted: AtomicU64,
    /// Total files quarantined
    pub files_quarantined: AtomicU64,
    /// Active threats
    pub active_threats: AtomicU32,
    /// Last scan time
    pub last_scan_time: AtomicU64,
    /// Defense enabled
    pub defense_enabled: AtomicBool,
}

impl DefenseStats {
    pub fn new() -> Self {
        DefenseStats {
            total_scans: AtomicU64::new(0),
            threats_detected: AtomicU64::new(0),
            threats_blocked: AtomicU64::new(0),
            attacks_intercepted: AtomicU64::new(0),
            files_quarantined: AtomicU64::new(0),
            active_threats: AtomicU32::new(0),
            last_scan_time: AtomicU64::new(0),
            defense_enabled: AtomicBool::new(true),
        }
    }
}

/// Defense configuration
#[repr(C)]
pub struct DefenseConfig {
    /// Enable real-time scanning
    pub realtime_scan: AtomicBool,
    /// Enable heuristic analysis
    pub heuristic_analysis: AtomicBool,
    /// Enable behavior monitoring
    pub behavior_monitor: AtomicBool,
    /// Enable network protection
    pub network_protection: AtomicBool,
    /// Enable intrusion detection
    pub intrusion_detection: AtomicBool,
    /// Maximum scan depth
    pub max_scan_depth: AtomicU32,
    /// Maximum file size to scan (bytes)
    pub max_file_size: AtomicU64,
    /// Quarantine path
    pub quarantine_path: [u8; 256],
    /// Auto-quarantine threshold
    pub auto_quarantine_level: AtomicU32,
    /// Action for each threat level
    pub level_actions: [AtomicU32; 5],
}

impl DefenseConfig {
    pub fn new() -> Self {
        DefenseConfig {
            realtime_scan: AtomicBool::new(true),
            heuristic_analysis: AtomicBool::new(true),
            behavior_monitor: AtomicBool::new(true),
            network_protection: AtomicBool::new(true),
            intrusion_detection: AtomicBool::new(true),
            max_scan_depth: AtomicU32::new(10),
            max_file_size: AtomicU64::new(100 * 1024 * 1024), // 100MB
            quarantine_path: [0; 256],
            auto_quarantine_level: AtomicU32::new(ThreatLevel::High as u32),
            level_actions: [
                AtomicU32::new(ThreatAction::Allow as u32),  // Safe
                AtomicU32::new(ThreatAction::Warn as u32),   // Low
                AtomicU32::new(ThreatAction::Warn as u32),   // Medium
                AtomicU32::new(ThreatAction::Quarantine as u32), // High
                AtomicU32::new(ThreatAction::Block as u32),  // Critical
            ],
        }
    }
}

/// Defense manager
pub struct DefenseManager {
    /// Statistics
    pub stats: DefenseStats,
    /// Configuration
    pub config: DefenseConfig,
    /// Threat database
    threat_db: alloc::sync::Arc<spin::Mutex<BTreeMap<u64, ThreatInfo>>>,
    /// Quarantine list
    quarantine: alloc::sync::Arc<spin::Mutex<Vec<QuarantineEntry>>>,
    /// Attack log
    attack_log: alloc::sync::Arc<spin::Mutex<Vec<AttackEvent>>>,
    /// Next threat ID
    next_threat_id: AtomicU64,
    /// Next scan ID
    next_scan_id: AtomicU64,
    /// Next attack ID
    next_attack_id: AtomicU64,
}

/// Quarantine entry
#[repr(C)]
pub struct QuarantineEntry {
    /// Entry ID
    pub id: u64,
    /// Original path
    pub original_path: [u8; 256],
    /// Quarantine path
    pub quarantine_path: [u8; 256],
    /// Threat info
    pub threat: ThreatInfo,
    /// Quarantine time
    pub quarantine_time: u64,
    /// File size
    pub file_size: u64,
    /// File hash
    pub file_hash: [u8; 32],
    /// Can restore
    pub can_restore: bool,
}

impl DefenseManager {
    /// Create new defense manager
    pub fn new() -> Self {
        DefenseManager {
            stats: DefenseStats::new(),
            config: DefenseConfig::new(),
            threat_db: alloc::sync::Arc::new(spin::Mutex::new(BTreeMap::new())),
            quarantine: alloc::sync::Arc::new(spin::Mutex::new(Vec::new())),
            attack_log: alloc::sync::Arc::new(spin::Mutex::new(Vec::new())),
            next_threat_id: AtomicU64::new(1),
            next_scan_id: AtomicU64::new(1),
            next_attack_id: AtomicU64::new(1),
        }
    }
    
    /// Initialize defense system
    pub fn init(&self) {
        log_info!("Security Defense Framework initialized");
        
        // Load threat database
        self.load_threat_database();
        
        // Start real-time protection
        if self.config.realtime_scan.load(Ordering::Acquire) {
            self.start_realtime_protection();
        }
        
        // Start behavior monitor
        if self.config.behavior_monitor.load(Ordering::Acquire) {
            self.start_behavior_monitor();
        }
    }
    
    /// Load threat database
    fn load_threat_database(&self) {
        // TODO: Load from file or network
        log_info!("Loading threat database...");
        
        // Add some known threats
        self.add_known_threats();
    }
    
    /// Add known threats to database
    fn add_known_threats(&self) {
        let known_threats = [
            ("EICAR.Test", ThreatCategory::Malware, ThreatLevel::Medium),
            ("Generic.Trojan", ThreatCategory::Trojan, ThreatLevel::High),
            ("Generic.Ransomware", ThreatCategory::Ransomware, ThreatLevel::Critical),
            ("Generic.Spyware", ThreatCategory::Spyware, ThreatLevel::Medium),
            ("Generic.Rootkit", ThreatCategory::Rootkit, ThreatLevel::Critical),
            ("Generic.Worm", ThreatCategory::Worm, ThreatLevel::High),
            ("Suspicious.Behavior", ThreatCategory::Suspicious, ThreatLevel::Low),
        ];
        
        let mut db = self.threat_db.lock();
        for (name, category, level) in known_threats {
            let id = self.next_threat_id.fetch_add(1, Ordering::AcqRel);
            let threat = ThreatInfo::new(id, name, category, level);
            db.insert(id, threat);
        }
    }
    
    /// Start real-time protection
    fn start_realtime_protection(&self) {
        log_info!("Starting real-time protection...");
        // TODO: Hook file operations for real-time scanning
    }
    
    /// Start behavior monitor
    fn start_behavior_monitor(&self) {
        log_info!("Starting behavior monitor...");
        // TODO: Monitor process behavior for suspicious activity
    }
    
    /// Scan file for threats
    pub fn scan_file(&self, path: &[u8]) -> Result<ScanResult, i32> {
        let scan_id = self.next_scan_id.fetch_add(1, Ordering::AcqRel);
        self.stats.total_scans.fetch_add(1, Ordering::AcqRel);
        
        log_info!("Starting scan {} for {:?}", scan_id, path);
        
        let mut result = ScanResult {
            scan_id,
            target: [0; 256],
            files_scanned: 1,
            bytes_scanned: 0,
            threats_found: 0,
            threats: core::array::from_fn(|_| None),
            duration_ms: 0,
            status: ScanStatus::InProgress,
        };
        
        // Copy path
        let len = path.len().min(255);
        result.target[..len].copy_from_slice(&path[..len]);
        
        // Perform scan
        let threats = self.detect_threats(path)?;
        
        // Fill result
        for (i, threat) in threats.iter().enumerate() {
            if i < 16 {
                result.threats[i] = Some(threat.clone());
            }
        }
        result.threats_found = threats.len() as u32;
        
        // Update stats
        if result.threats_found > 0 {
            self.stats.threats_detected.fetch_add(result.threats_found as u64, Ordering::AcqRel);
            self.stats.active_threats.fetch_add(result.threats_found, Ordering::AcqRel);
        }
        
        result.status = ScanStatus::Completed;
        self.stats.last_scan_time.store(
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { crate::kernel::time::get_time_ms() },
            Ordering::Release
        );
        
        Ok(result)
    }
    
    /// Scan directory recursively
    pub fn scan_directory(&self, path: &[u8], max_depth: u32) -> Result<ScanResult, i32> {
        let scan_id = self.next_scan_id.fetch_add(1, Ordering::AcqRel);
        self.stats.total_scans.fetch_add(1, Ordering::AcqRel);
        
        log_info!("Starting directory scan {} for {:?}", scan_id, path);
        
        // TODO: Implement recursive directory scanning
        let result = ScanResult {
            scan_id,
            target: [0; 256],
            files_scanned: 0,
            bytes_scanned: 0,
            threats_found: 0,
            threats: core::array::from_fn(|_| None),
            duration_ms: 0,
            status: ScanStatus::Completed,
        };
        
        Ok(result)
    }
    
    /// Detect threats in file
    fn detect_threats(&self, path: &[u8]) -> Result<Vec<ThreatInfo>, i32> {
        let mut threats = Vec::new();
        
        // Signature-based detection
        if let Some(t) = self.signature_scan(path)? {
            threats.push(t);
        }
        
        // Heuristic analysis
        if self.config.heuristic_analysis.load(Ordering::Acquire) {
            if let Some(t) = self.heuristic_scan(path)? {
                threats.push(t);
            }
        }
        
        Ok(threats)
    }
    
    /// Signature-based scan
    fn signature_scan(&self, path: &[u8]) -> Result<Option<ThreatInfo>, i32> {
        // TODO: Read file and check signatures
        // For now, check file name patterns
        let suspicious_patterns = [
            b"eicar" as &[u8],
            b"malware",
            b"trojan",
            b"virus",
            b"ransomware",
        ];
        
        for pattern in suspicious_patterns {
            if self.contains_pattern(path, pattern) {
                let db = self.threat_db.lock();
                if let Some((_, threat)) = db.iter().next() {
                    return Ok(Some(threat.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Heuristic scan
    fn heuristic_scan(&self, path: &[u8]) -> Result<Option<ThreatInfo>, i32> {
        // TODO: Implement heuristic analysis
        // Check for suspicious characteristics:
        // - High entropy (packed/encrypted)
        // - Unusual section names
        // - Suspicious imports
        // - Anomalous behavior patterns
        
        Ok(None)
    }
    
    /// Check if path contains pattern
    fn contains_pattern(&self, path: &[u8], pattern: &[u8]) -> bool {
        if pattern.len() > path.len() {
            return false;
        }
        
        for i in 0..=path.len() - pattern.len() {
            if &path[i..i + pattern.len()] == pattern {
                return true;
            }
        }
        
        false
    }
    
    /// Handle detected threat
    pub fn handle_threat(&self, threat: &ThreatInfo, path: &[u8]) -> ThreatAction {
        let action = self.get_action_for_level(threat.level);
        
        match action {
            ThreatAction::Quarantine => {
                if self.quarantine_file(path, threat).is_ok() {
                    self.stats.files_quarantined.fetch_add(1, Ordering::AcqRel);
                }
            }
            ThreatAction::Block => {
                self.stats.threats_blocked.fetch_add(1, Ordering::AcqRel);
            }
            ThreatAction::Kill => {
                // TODO: Kill the process
                self.stats.threats_blocked.fetch_add(1, Ordering::AcqRel);
            }
            _ => {}
        }
        
        action
    }
    
    /// Get action for threat level
    fn get_action_for_level(&self, level: ThreatLevel) -> ThreatAction {
        let idx = level as usize;
        if idx < 5 {
            match self.config.level_actions[idx].load(Ordering::Acquire) {
                0 => ThreatAction::Allow,
                1 => ThreatAction::Warn,
                2 => ThreatAction::Quarantine,
                3 => ThreatAction::Block,
                4 => ThreatAction::Delete,
                5 => ThreatAction::Kill,
                _ => ThreatAction::Warn,
            }
        } else {
            ThreatAction::Warn
        }
    }
    
    /// Quarantine file
    fn quarantine_file(&self, path: &[u8], threat: &ThreatInfo) -> Result<(), i32> {
        let id = self.next_threat_id.fetch_add(1, Ordering::AcqRel);
        
        let mut entry = QuarantineEntry {
            id,
            original_path: [0; 256],
            quarantine_path: [0; 256],
            threat: threat.clone(),
            // SAFETY: unsafe block required for low-level memory or hardware access
            quarantine_time: unsafe { crate::kernel::time::get_time_ms() },
            file_size: 0,
            file_hash: [0; 32],
            can_restore: true,
        };
        
        // Copy paths
        let len = path.len().min(255);
        entry.original_path[..len].copy_from_slice(&path[..len]);
        
        // Generate quarantine path
        let qpath = b"/quarantine/";
        let qpath_len = qpath.len();
        entry.quarantine_path[..qpath_len].copy_from_slice(qpath);
        
        // TODO: Move file to quarantine
        
        self.quarantine.lock().push(entry);
        
        Ok(())
    }
    
    /// Restore file from quarantine
    pub fn restore_from_quarantine(&self, id: u64) -> Result<(), i32> {
        let mut quarantine = self.quarantine.lock();
        
        let idx = quarantine.iter().position(|e| e.id == id).ok_or(-2)?;
        let entry = &quarantine[idx];
        
        if !entry.can_restore {
            return Err(-1);
        }
        
        // TODO: Move file back to original location
        
        quarantine.remove(idx);
        Ok(())
    }
    
    /// Intercept attack
    pub fn intercept_attack(&self, attack: AttackType, source: &[u8], target_pid: u32) -> ThreatAction {
        let id = self.next_attack_id.fetch_add(1, Ordering::AcqRel);
        
        let level = self.assess_attack_level(attack);
        let action = self.get_action_for_level(level);
        
        let mut event = AttackEvent {
            id,
            attack_type: attack,
            source_ip: [0; 16],
            source_port: 0,
            target_pid,
            // SAFETY: unsafe block required for low-level memory or hardware access
            timestamp: unsafe { crate::kernel::time::get_time_ms() },
            level,
            action,
            details: [0; 512],
            blocked: AtomicBool::new(action == ThreatAction::Block),
        };
        
        // Copy source IP
        let len = source.len().min(16);
        event.source_ip[..len].copy_from_slice(&source[..len]);
        
        // Log attack
        self.attack_log.lock().push(event);
        
        // Update stats
        self.stats.attacks_intercepted.fetch_add(1, Ordering::AcqRel);
        
        if action == ThreatAction::Block {
            self.stats.threats_blocked.fetch_add(1, Ordering::AcqRel);
            log_warn!("Attack blocked: {:?} from {:?}", attack, source);
        }
        
        action
    }
    
    /// Assess attack level
    fn assess_attack_level(&self, attack: AttackType) -> ThreatLevel {
        match attack {
            AttackType::BufferOverflow => ThreatLevel::Critical,
            AttackType::SqlInjection => ThreatLevel::High,
            AttackType::XssAttack => ThreatLevel::Medium,
            AttackType::CsrfAttack => ThreatLevel::Medium,
            AttackType::PathTraversal => ThreatLevel::High,
            AttackType::CodeInjection => ThreatLevel::Critical,
            AttackType::DdosAttack => ThreatLevel::Critical,
            AttackType::BruteForce => ThreatLevel::Medium,
            AttackType::PortScan => ThreatLevel::Low,
            AttackType::PrivilegeEscalation => ThreatLevel::Critical,
            AttackType::Unknown => ThreatLevel::Medium,
        }
    }
    
    /// Check process behavior
    pub fn check_process_behavior(&self, pid: u32, action: ProcessAction) -> ThreatLevel {
        // Suspicious behaviors
        match action {
            ProcessAction::ModifySystemFile => ThreatLevel::High,
            ProcessAction::InjectCode => ThreatLevel::Critical,
            ProcessAction::HookSystemCall => ThreatLevel::Critical,
            ProcessAction::AccessCredential => ThreatLevel::High,
            ProcessAction::NetworkConnect => ThreatLevel::Low,
            ProcessAction::FileCreate => ThreatLevel::Safe,
            ProcessAction::FileWrite => ThreatLevel::Safe,
            ProcessAction::ProcessCreate => ThreatLevel::Low,
            ProcessAction::Unknown => ThreatLevel::Safe,
        }
    }
    
    /// Enable/disable defense
    pub fn set_defense_enabled(&self, enabled: bool) {
        self.stats.defense_enabled.store(enabled, Ordering::Release);
        if enabled {
            log_info!("Security defense enabled");
        } else {
            log_warn!("Security defense disabled");
        }
    }
    
    /// Get defense status
    pub fn is_defense_enabled(&self) -> bool {
        self.stats.defense_enabled.load(Ordering::Acquire)
    }
    
    /// Get threat count
    pub fn get_threat_count(&self) -> u32 {
        self.stats.active_threats.load(Ordering::Acquire)
    }
    
    /// Get quarantine count
    pub fn get_quarantine_count(&self) -> usize {
        self.quarantine.lock().len()
    }
    
    /// Clear attack log
    pub fn clear_attack_log(&self) {
        self.attack_log.lock().clear();
    }
}

/// Process action for behavior monitoring
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAction {
    /// Unknown action
    Unknown = 0,
    /// Create file
    FileCreate = 1,
    /// Write file
    FileWrite = 2,
    /// Modify system file
    ModifySystemFile = 3,
    /// Create process
    ProcessCreate = 4,
    /// Inject code
    InjectCode = 5,
    /// Hook system call
    HookSystemCall = 6,
    /// Access credential
    AccessCredential = 7,
    /// Network connect
    NetworkConnect = 8,
}

impl Default for DefenseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global defense manager
static DEFENSE_MANAGER: crate::sync_oncelock::OnceLock<DefenseManager> = crate::sync_oncelock::OnceLock::new();

/// Get defense manager
pub fn get_defense_manager() -> &'static mut DefenseManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { DEFENSE_MANAGER.assume_init_mut() }
}

/// Initialize defense system
pub fn init_defense() {
    // SAFETY: DEFENSE_MANAGER is only written here during init
    unsafe { DEFENSE_MANAGER.write(DefenseManager::new()); }
    let mgr = get_defense_manager();
    mgr.init();
}

/// Quick scan file
pub fn quick_scan(path: &[u8]) -> Result<ScanResult, i32> {
    get_defense_manager().scan_file(path)
}

/// Check if file is safe
pub fn is_file_safe(path: &[u8]) -> bool {
    match quick_scan(path) {
        Ok(result) => result.threats_found == 0,
        Err(_) => false,
    }
}
