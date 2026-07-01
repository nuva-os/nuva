/* * Nuva OS - Sessionmanagementadministration
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
use super::user::{Uid, Gid};

/// Session ID Type
pub type SessionId = u32;

/// ProcessGroup ID Type
pub type Pgid = u32;

/// MaxSessionnumber
pub const MAX_SESSIONS: usize = 64;

/// SessionState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
 /// active
 Active = 0,
 /// emptyidle
 Idle = 1,
 /// alreadyEnd
 Terminated = 2,
}

/// Sessionstruct
pub struct Session {
 /// Session ID
 pub sid: SessionId,
 /// User ID
 pub uid: Uid,
 /// mainGroup ID
 pub gid: Gid,
 /// Controlendend
 pub controlling_tty: AtomicU32,
 /// prefixProcessGroup
 pub foreground_pgid: AtomicU32,
 /// ProcessGroupnumber
 pub process_group_count: AtomicU32,
 /// Processnumber
 pub process_count: AtomicU32,
 /// CreateTime
 pub create_time: AtomicU64,
 /// State
 pub state: AtomicU32,
}

impl Session {
 /// CreatenewSession
 pub fn new(sid: SessionId, uid: Uid, gid: Gid) -> Self {
 Session {
 sid,
 uid,
 gid,
 controlling_tty: AtomicU32::new(0),
 foreground_pgid: AtomicU32::new(0),
 process_group_count: AtomicU32::new(0),
 process_count: AtomicU32::new(0),
 create_time: AtomicU64::new(0),
 state: AtomicU32::new(SessionState::Active as u32),
 }
 }
 
 /// GetState
 pub fn get_state(&self) -> SessionState {
 match self.state.load(Ordering::Acquire) {
 0 => SessionState::Active,
 1 => SessionState::Idle,
 2 => SessionState::Terminated,
 _ => SessionState::Active,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: SessionState) {
 self.state.store(state as u32, Ordering::Release);
 }
 
 /// SetControlendend
 pub fn set_controlling_tty(&self, tty: u32) {
 self.controlling_tty.store(tty, Ordering::Release);
 }
 
 /// GetControlendend
 pub fn get_controlling_tty(&self) -> u32 {
 self.controlling_tty.load(Ordering::Acquire)
 }
 
 /// SetprefixProcessGroup
 pub fn set_foreground_pgid(&self, pgid: Pgid) {
 self.foreground_pgid.store(pgid, Ordering::Release);
 }
 
 /// GetprefixProcessGroup
 pub fn get_foreground_pgid(&self) -> Pgid {
 self.foreground_pgid.load(Ordering::Acquire)
 }
 
 /// increasePlusProcess
 pub fn add_process(&self) {
 self.process_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// MinusfewProcess
 pub fn remove_process(&self) {
 let count = self.process_count.fetch_sub(1, Ordering::AcqRel);
 if count == 1 {
 self.set_state(SessionState::Terminated);
 }
 }
}

/// ProcessGroupstruct
pub struct ProcessGroup {
 /// ProcessGroup ID
 pub pgid: Pgid,
 /// Session ID
 pub sid: SessionId,
 /// Processnumber
 pub process_count: AtomicU32,
}

impl ProcessGroup {
 /// CreatenewProcessGroup
 pub fn new(pgid: Pgid, sid: SessionId) -> Self {
 ProcessGroup {
 pgid,
 sid,
 process_count: AtomicU32::new(0),
 }
 }
 
 /// increasePlusProcess
 pub fn add_process(&self) {
 self.process_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// MinusfewProcess
 pub fn remove_process(&self) {
 self.process_count.fetch_sub(1, Ordering::AcqRel);
 }
}

/// SessionManager
pub struct SessionManager {
 /// SessionArray
 sessions: [Option<Session>; MAX_SESSIONS],
 /// Session count
 session_count: AtomicU32,
 /// CurrentSession ID
 current_sid: AtomicU32,
 /// NextSession ID
 next_sid: AtomicU32,
}

impl SessionManager {
 pub const fn new() -> Self {
 SessionManager {
 sessions: [None; MAX_SESSIONS],
 session_count: AtomicU32::new(0),
 current_sid: AtomicU32::new(0),
 next_sid: AtomicU32::new(1),
 }
 }
 
 /// Initialize
 pub fn init(&self) {
 log_info!("Session manager initialized");
 }
 
 /// CreateSession
 pub fn create_session(&mut self, uid: Uid, gid: Gid) -> Option<SessionId> {
 let sid = self.next_sid.fetch_add(1, Ordering::AcqRel);
 
 if sid as usize >= MAX_SESSIONS {
 return None;
 }
 
 let session = Session::new(sid, uid, gid);
 self.sessions[sid as usize] = Some(session);
 self.session_count.fetch_add(1, Ordering::AcqRel);
 
 log_info!("Created session: sid={}, uid={}", sid, uid);
 
 Some(sid)
 }
 
 /// FindSession
 pub fn find_session(&self, sid: SessionId) -> Option<&Session> {
 if sid as usize >= MAX_SESSIONS {
 return None;
 }
 self.sessions[sid as usize].as_ref()
 }
 
 /// FindSession (canchange)
 pub fn find_session_mut(&mut self, sid: SessionId) -> Option<&mut Session> {
 if sid as usize >= MAX_SESSIONS {
 return None;
 }
 self.sessions[sid as usize].as_mut()
 }
 
 /// GetCurrentSession
 pub fn get_current_sid(&self) -> SessionId {
 self.current_sid.load(Ordering::Acquire)
 }
 
 /// SetCurrentSession
 pub fn set_current_sid(&self, sid: SessionId) {
 self.current_sid.store(sid, Ordering::Release);
 }
 
 /// EndSession
 pub fn terminate_session(&mut self, sid: SessionId) -> bool {
 if let Some(session) = self.find_session_mut(sid) {
 session.set_state(SessionState::Terminated);
 log_info!("Terminated session: sid={}", sid);
 return true;
 }
 false
 }
 
 /// DeleteSession
 pub fn delete_session(&mut self, sid: SessionId) -> bool {
 if sid as usize >= MAX_SESSIONS {
 return false;
 }
 
 if self.sessions[sid as usize].take().is_some() {
 self.session_count.fetch_sub(1, Ordering::AcqRel);
 log_info!("Deleted session: sid={}", sid);
 return true;
 }
 
 false
 }
 
 /// GetSession count
 pub fn get_session_count(&self) -> u32 {
 self.session_count.load(Ordering::Acquire)
 }
}

/// GlobalSessionManager
static SESSION_MANAGER: crate::sync_oncelock::OnceLock<SessionManager> = crate::sync_oncelock::OnceLock::new();

pub fn session_manager() -> &'static SessionManager {
    SESSION_MANAGER.get_or_init(SessionManager::new)
}

pub fn init_session_manager() {
 let manager = session_manager();
 manager.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_constants() {
 assert_eq!(MAX_SESSIONS, 64);
 }

 #[test]
 fn test_session_state_values() {
 assert_eq!(SessionState::Active as u32, 0);
 assert_eq!(SessionState::Idle as u32, 1);
 assert_eq!(SessionState::Terminated as u32, 2);
 }

 #[test]
 fn test_session_new() {
 let session = Session::new(1, 100, 100);

 assert_eq!(session.sid, 1);
 assert_eq!(session.uid, 100);
 assert_eq!(session.gid, 100);
 assert_eq!(session.get_state(), SessionState::Active);
 assert_eq!(session.get_controlling_tty(), 0);
 assert_eq!(session.get_foreground_pgid(), 0);
 }

 #[test]
 fn test_session_state_transitions() {
 let session = Session::new(1, 0, 0);

 assert_eq!(session.get_state(), SessionState::Active);

 session.set_state(SessionState::Idle);
 assert_eq!(session.get_state(), SessionState::Idle);

 session.set_state(SessionState::Terminated);
 assert_eq!(session.get_state(), SessionState::Terminated);
 }

 #[test]
 fn test_session_controlling_tty() {
 let session = Session::new(1, 0, 0);

 assert_eq!(session.get_controlling_tty(), 0);

 session.set_controlling_tty(42);
 assert_eq!(session.get_controlling_tty(), 42);
 }

 #[test]
 fn test_session_foreground_pgid() {
 let session = Session::new(1, 0, 0);

 assert_eq!(session.get_foreground_pgid(), 0);

 session.set_foreground_pgid(100);
 assert_eq!(session.get_foreground_pgid(), 100);
 }

 #[test]
 fn test_session_process_count() {
 let session = Session::new(1, 0, 0);

 assert_eq!(session.process_count.load(Ordering::Relaxed), 0);

 session.add_process();
 assert_eq!(session.process_count.load(Ordering::Relaxed), 1);

 session.add_process();
 session.add_process();
 assert_eq!(session.process_count.load(Ordering::Relaxed), 3);

 session.remove_process();
 assert_eq!(session.process_count.load(Ordering::Relaxed), 2);
 }

 #[test]
 fn test_session_terminate_on_last_process() {
 let session = Session::new(1, 0, 0);

 session.add_process();
 session.add_process();
 assert_eq!(session.get_state(), SessionState::Active);

 session.remove_process();
 assert_eq!(session.get_state(), SessionState::Active);

 session.remove_process();
 assert_eq!(session.get_state(), SessionState::Terminated);
 }

 #[test]
 fn test_session_process_group_count() {
 let session = Session::new(1, 0, 0);

 assert_eq!(session.process_group_count.load(Ordering::Relaxed), 0);

 session.process_group_count.fetch_add(1, Ordering::Relaxed);
 assert_eq!(session.process_group_count.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_process_group_new() {
 let pg = ProcessGroup::new(100, 1);

 assert_eq!(pg.pgid, 100);
 assert_eq!(pg.sid, 1);
 assert_eq!(pg.process_count.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_process_group_process_count() {
 let pg = ProcessGroup::new(100, 1);

 assert_eq!(pg.process_count.load(Ordering::Relaxed), 0);

 pg.add_process();
 assert_eq!(pg.process_count.load(Ordering::Relaxed), 1);

 pg.add_process();
 assert_eq!(pg.process_count.load(Ordering::Relaxed), 2);

 pg.remove_process();
 assert_eq!(pg.process_count.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_session_manager_new() {
 let mgr = SessionManager::new();

 assert_eq!(mgr.get_session_count(), 0);
 assert_eq!(mgr.get_current_sid(), 0);
 }

 #[test]
 fn test_session_manager_create_session() {
 let mut mgr = SessionManager::new();

 let sid = mgr.create_session(100, 100);
 assert!(sid.is_some());
 assert_eq!(sid.unwrap(), 1);
 assert_eq!(mgr.get_session_count(), 1);

 let session = mgr.find_session(1);
 assert!(session.is_some());
 assert_eq!(session.unwrap().uid, 100);
 }

 #[test]
 fn test_session_manager_multiple_sessions() {
 let mut mgr = SessionManager::new();

 let sid1 = mgr.create_session(100, 100);
 let sid2 = mgr.create_session(200, 200);
 let sid3 = mgr.create_session(300, 300);

 assert_eq!(sid1.unwrap(), 1);
 assert_eq!(sid2.unwrap(), 2);
 assert_eq!(sid3.unwrap(), 3);
 assert_eq!(mgr.get_session_count(), 3);
 }

 #[test]
 fn test_session_manager_find_session() {
 let mut mgr = SessionManager::new();
 mgr.create_session(100, 100);

 let session = mgr.find_session(1);
 assert!(session.is_some());

 let not_found = mgr.find_session(99);
 assert!(not_found.is_none());
 }

 #[test]
 fn test_session_manager_find_session_mut() {
 let mut mgr = SessionManager::new();
 mgr.create_session(100, 100);

 let session = mgr.find_session_mut(1);
 assert!(session.is_some());

 let session = session.unwrap();
 session.set_foreground_pgid(42);
 assert_eq!(session.get_foreground_pgid(), 42);
 }

 #[test]
 fn test_session_manager_current_sid() {
 let mgr = SessionManager::new();

 assert_eq!(mgr.get_current_sid(), 0);

 mgr.set_current_sid(5);
 assert_eq!(mgr.get_current_sid(), 5);
 }

 #[test]
 fn test_session_manager_terminate_session() {
 let mut mgr = SessionManager::new();
 mgr.create_session(100, 100);

 let result = mgr.terminate_session(1);
 assert!(result);

 let session = mgr.find_session(1);
 assert!(session.is_some());
 assert_eq!(session.unwrap().get_state(), SessionState::Terminated);
 }

 #[test]
 fn test_session_manager_terminate_nonexistent() {
 let mut mgr = SessionManager::new();

 let result = mgr.terminate_session(99);
 assert!(!result);
 }

 #[test]
 fn test_session_manager_delete_session() {
 let mut mgr = SessionManager::new();
 mgr.create_session(100, 100);

 assert_eq!(mgr.get_session_count(), 1);

 let result = mgr.delete_session(1);
 assert!(result);
 assert_eq!(mgr.get_session_count(), 0);

 let session = mgr.find_session(1);
 assert!(session.is_none());
 }

 #[test]
 fn test_session_manager_delete_nonexistent() {
 let mut mgr = SessionManager::new();

 let result = mgr.delete_session(99);
 assert!(!result);
 }

 #[test]
 fn test_session_manager_max_sessions() {
 let mut mgr = SessionManager::new();

 // CreateMaxcount Session
 for i in 0..MAX_SESSIONS - 1 {
 let result = mgr.create_session(0, 0);
 if result.is_none() {
 break;
 }
 }

 // shouldthereachtoMaxSessionnumber
 assert_eq!(mgr.get_session_count() as usize, MAX_SESSIONS - 1);

 // againCreateaitemshouldtheFailure
 let result = mgr.create_session(0, 0);
 assert!(result.is_none());
 }

 #[test]
 fn test_session_create_time() {
 let session = Session::new(1, 0, 0);

 assert_eq!(session.create_time.load(Ordering::Relaxed), 0);

 session.create_time.store(12345, Ordering::Relaxed);
 assert_eq!(session.create_time.load(Ordering::Relaxed), 12345);
 }
}