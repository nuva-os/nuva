/*
 * Nuva OS - TCP Congestion Control
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

//! TCP Congestion Control
/*!*/
//! Implements Reno, CUBIC, and BBR congestion control algorithms.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Congestion control state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CcState {
    /// Slow start
    SlowStart = 0,

    /// Congestion avoidance
    CongestionAvoidance = 1,

    /// Fast retransmit
    FastRetransmit = 2,

    /// Fast recovery
    FastRecovery = 3,
}

/// TCP Reno Congestion Control
pub struct RenoCongestionControl {
    /// Congestion window (bytes)
    pub cwnd: AtomicU32,

    /// Slow start threshold (bytes)
    pub ssthresh: AtomicU32,

    /// Congestion state
    pub state: AtomicU32,

    /// Duplicate ACK count
    pub dup_ack_count: AtomicU32,

    /// Last ACK number
    pub last_ack: AtomicU32,

    /// Recovery point
    pub recover: AtomicU32,

    /// Smoothed RTT (ms)
    pub srtt: AtomicU32,

    /// RTT variance (ms)
    pub rttvar: AtomicU32,

    /// Retransmission timeout (ms)
    pub rto: AtomicU32,

    /// Minimum RTO (ms)
    pub min_rto: u32,

    /// Maximum RTO (ms)
    pub max_rto: u32,
}

impl RenoCongestionControl {
    pub const fn new() -> Self {
        Self {
            cwnd: AtomicU32::new(1460), // 1 MSS
            ssthresh: AtomicU32::new(65535),
            state: AtomicU32::new(CcState::SlowStart as u32),
            dup_ack_count: AtomicU32::new(0),
            last_ack: AtomicU32::new(0),
            recover: AtomicU32::new(0),
            srtt: AtomicU32::new(0),
            rttvar: AtomicU32::new(0),
            rto: AtomicU32::new(1000),
            min_rto: 200,
            max_rto: 60000,
        }
    }

    /// Get congestion state
    pub fn get_state(&self) -> CcState {
        match self.state.load(Ordering::Acquire) {
            0 => CcState::SlowStart,
            1 => CcState::CongestionAvoidance,
            2 => CcState::FastRetransmit,
            3 => CcState::FastRecovery,
            _ => CcState::SlowStart,
        }
    }

    /// Set congestion state
    fn set_state(&self, state: CcState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Handle new ACK
    pub fn on_new_ack(&self, ack: u32, mss: u32) {
        let state = self.get_state();
        let mut cwnd = self.cwnd.load(Ordering::Acquire);

        match state {
            CcState::SlowStart => {
                // Exponential growth
                cwnd += mss;
                self.cwnd.store(cwnd, Ordering::Release);

                // Check if reached ssthresh
                if cwnd >= self.ssthresh.load(Ordering::Acquire) {
                    self.set_state(CcState::CongestionAvoidance);
                }
            }
            CcState::CongestionAvoidance => {
                // Linear growth: cwnd += MSS * MSS / cwnd
                cwnd += (mss * mss) / cwnd;
                self.cwnd.store(cwnd, Ordering::Release);
            }
            CcState::FastRecovery => {
                // Exit fast recovery
                let ssthresh = self.ssthresh.load(Ordering::Acquire);
                self.cwnd.store(ssthresh, Ordering::Release);
                self.dup_ack_count.store(0, Ordering::Release);
                self.set_state(CcState::CongestionAvoidance);
            }
            _ => {}
        }

        self.last_ack.store(ack, Ordering::Release);
    }

    /// Handle duplicate ACK
    pub fn on_dup_ack(&self, ack: u32, mss: u32) {
        let state = self.get_state();
        let dup_count = self.dup_ack_count.fetch_add(1, Ordering::AcqRel) + 1;

        match state {
            CcState::SlowStart | CcState::CongestionAvoidance => {
                // Check for fast retransmit (3 duplicate ACKs)
                if dup_count == 3 {
                    // Fast retransmit
                    let cwnd = self.cwnd.load(Ordering::Acquire);
                    let ssthresh = cwnd / 2;

                    self.ssthresh.store(ssthresh, Ordering::Release);
                    self.cwnd.store(ssthresh + 3 * mss, Ordering::Release);
                    self.recover.store(ack, Ordering::Release);
                    self.set_state(CcState::FastRecovery);
                }
            }
            CcState::FastRecovery => {
                // Inflate window
                let cwnd = self.cwnd.load(Ordering::Acquire) + mss;
                self.cwnd.store(cwnd, Ordering::Release);
            }
            _ => {}
        }
    }

    /// Handle timeout
    pub fn on_timeout(&self) {
        // Reset to slow start
        let cwnd = self.cwnd.load(Ordering::Acquire);
        let ssthresh = cwnd / 2;

        self.ssthresh.store(ssthresh.max(2 * 1460), Ordering::Release);
        self.cwnd.store(1460, Ordering::Release);
        self.dup_ack_count.store(0, Ordering::Release);
        self.set_state(CcState::SlowStart);

        // Exponential backoff
        let rto = self.rto.load(Ordering::Acquire);
        let new_rto = (rto * 2).min(self.max_rto);
        self.rto.store(new_rto, Ordering::Release);
    }

    /// Update RTT estimation
    pub fn update_rtt(&self, rtt: u32) {
        let srtt = self.srtt.load(Ordering::Acquire);
        let rttvar = self.rttvar.load(Ordering::Acquire);

        if srtt == 0 {
            // First measurement
            self.srtt.store(rtt, Ordering::Release);
            self.rttvar.store(rtt / 2, Ordering::Release);
        } else {
            // Jacobson/Karels algorithm
            let delta = if rtt > srtt { rtt - srtt } else { srtt - rtt };
            let new_rttvar = (3 * rttvar + delta) / 4;
            let new_srtt = (7 * srtt + rtt) / 8;

            self.rttvar.store(new_rttvar, Ordering::Release);
            self.srtt.store(new_srtt, Ordering::Release);
        }

        // Update RTO
        let srtt = self.srtt.load(Ordering::Acquire);
        let rttvar = self.rttvar.load(Ordering::Acquire);
        let rto = (srtt + 4 * rttvar).max(self.min_rto).min(self.max_rto);
        self.rto.store(rto, Ordering::Release);
    }

    /// Get congestion window
    pub fn get_cwnd(&self) -> u32 {
        self.cwnd.load(Ordering::Acquire)
    }

    /// Get RTO
    pub fn get_rto(&self) -> u32 {
        self.rto.load(Ordering::Acquire)
    }
}

/// TCP CUBIC Congestion Control
pub struct CubicCongestionControl {
    /// Congestion window
    pub cwnd: AtomicU32,

    /// Slow start threshold
    pub ssthresh: AtomicU32,

    /// Window max (W_max)
    pub w_max: AtomicU32,

    /// Time since last congestion (K)
    pub epoch_start: AtomicU64,

    /// CUBIC parameter C
    pub c: u32,

    /// CUBIC parameter beta
    pub beta: u32,

    /// MSS
    pub mss: u32,
}

impl CubicCongestionControl {
    pub const fn new() -> Self {
        Self {
            cwnd: AtomicU32::new(1460),
            ssthresh: AtomicU32::new(65535),
            w_max: AtomicU32::new(0),
            epoch_start: AtomicU64::new(0),
            c: 410, // 0.4 * 1024
            beta: 717, // 0.7 * 1024
            mss: 1460,
        }
    }

    /// Calculate CUBIC window
    fn cubic_window(&self, t: u64) -> u32 {
        let w_max = self.w_max.load(Ordering::Acquire);
        let c = self.c;

        // W_cubic(t) = C * t^3 + W_max
        // Using fixed-point arithmetic
        let t3 = t * t * t;
        let cubic_term = (c * t3 as u32) / 1024;

        w_max + cubic_term
    }

    /// Handle new ACK
    pub fn on_new_ack(&self, timestamp: u64) {
        let epoch_start = self.epoch_start.load(Ordering::Acquire);

        if epoch_start == 0 {
            self.epoch_start.store(timestamp, Ordering::Release);
        }

        let t = timestamp - epoch_start;
        let w_cubic = self.cubic_window(t);

        // TCP-friendly region
        let cwnd = self.cwnd.load(Ordering::Acquire);
        let w_max = self.w_max.load(Ordering::Acquire);

        if w_cubic < cwnd {
            // TCP region
            self.cwnd.store(cwnd + self.mss, Ordering::Release);
        } else {
            // CUBIC region
            self.cwnd.store(w_cubic, Ordering::Release);
        }
    }

    /// Handle loss
    pub fn on_loss(&self) {
        let cwnd = self.cwnd.load(Ordering::Acquire);
        let beta = self.beta;

        // W_max = cwnd
        self.w_max.store(cwnd, Ordering::Release);

        // ssthresh = cwnd * beta
        let ssthresh = (cwnd as u64 * beta as u64 / 1024) as u32;
        self.ssthresh.store(ssthresh, Ordering::Release);
        self.cwnd.store(ssthresh, Ordering::Release);

        // Reset epoch
        self.epoch_start.store(0, Ordering::Release);
    }

    /// Get congestion window
    pub fn get_cwnd(&self) -> u32 {
        self.cwnd.load(Ordering::Acquire)
    }
}

/// BBR Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BbrMode {
    Startup = 0,
    Drain = 1,
    ProbeBw = 2,
    ProbeRtt = 3,
}

/// TCP BBR Congestion Control
pub struct BbrCongestionControl {
    /// BBR mode
    pub mode: AtomicU32,

    /// Estimated bandwidth (bytes/sec)
    pub bw: AtomicU32,

    /// Minimum RTT (us)
    pub min_rtt: AtomicU32,

    /// Minimum RTT timestamp
    pub min_rtt_stamp: AtomicU64,

    /// Pacing rate (bytes/sec)
    pub pacing_rate: AtomicU32,

    /// Send cwnd
    pub send_cwnd: AtomicU32,

    /// Target cwnd
    pub target_cwnd: AtomicU32,

    /// BDP
    pub bdp: AtomicU32,

    /// Round count
    pub round_count: AtomicU32,

    /// RTT count for ProbeRTT
    pub rtt_cnt: AtomicU32,

    /// ProbeRTT done
    pub probe_rtt_done: AtomicU32,
}

impl BbrCongestionControl {
    pub const fn new() -> Self {
        Self {
            mode: AtomicU32::new(BbrMode::Startup as u32),
            bw: AtomicU32::new(0),
            min_rtt: AtomicU32::new(u32::MAX),
            min_rtt_stamp: AtomicU64::new(0),
            pacing_rate: AtomicU32::new(0),
            send_cwnd: AtomicU32::new(1460),
            target_cwnd: AtomicU32::new(0),
            bdp: AtomicU32::new(0),
            round_count: AtomicU32::new(0),
            rtt_cnt: AtomicU32::new(0),
            probe_rtt_done: AtomicU32::new(0),
        }
    }

    /// Get BBR mode
    pub fn get_mode(&self) -> BbrMode {
        match self.mode.load(Ordering::Acquire) {
            0 => BbrMode::Startup,
            1 => BbrMode::Drain,
            2 => BbrMode::ProbeBw,
            3 => BbrMode::ProbeRtt,
            _ => BbrMode::Startup,
        }
    }

    /// Update bandwidth estimation
    pub fn update_bw(&self, delivered: u32, interval_us: u64) {
        if interval_us > 0 {
            // BW = delivered / interval
            let bw = (delivered as u64 * 1_000_000 / interval_us) as u32;

            // Update max bandwidth
            let current_bw = self.bw.load(Ordering::Acquire);
            if bw > current_bw {
                self.bw.store(bw, Ordering::Release);
            }
        }
    }

    /// Update minimum RTT
    pub fn update_min_rtt(&self, rtt: u32, timestamp: u64) {
        let min_rtt = self.min_rtt.load(Ordering::Acquire);

        if rtt < min_rtt || timestamp - self.min_rtt_stamp.load(Ordering::Acquire) > 10_000_000 {
            self.min_rtt.store(rtt, Ordering::Release);
            self.min_rtt_stamp.store(timestamp, Ordering::Release);
        }
    }

    /// Calculate BDP
    pub fn calculate_bdp(&self) -> u32 {
        let bw = self.bw.load(Ordering::Acquire);
        let min_rtt = self.min_rtt.load(Ordering::Acquire);

        if min_rtt == u32::MAX || min_rtt == 0 {
            return 0;
        }

        // BDP = BW * RTT
        (bw as u64 * min_rtt as u64 / 1_000_000) as u32
    }

    /// Update pacing rate
    pub fn update_pacing_rate(&self) {
        let bw = self.bw.load(Ordering::Acquire);
        // Pacing rate = 1.25 * BW
        let pacing_rate = (bw as u64 * 5 / 4) as u32;
        self.pacing_rate.store(pacing_rate, Ordering::Release);
    }

    /// Set send cwnd
    pub fn set_send_cwnd(&self) {
        let bdp = self.calculate_bdp();
        self.bdp.store(bdp, Ordering::Release);

        // Send cwnd = BDP * gain
        let send_cwnd = (bdp as u64 * 5 / 4) as u32;
        self.send_cwnd.store(send_cwnd.max(1460), Ordering::Release);
    }

    /// Handle ACK
    pub fn on_ack(&self, delivered: u32, rtt: u32, timestamp: u64) {
        // Update measurements
        self.update_bw(delivered, rtt as u64);
        self.update_min_rtt(rtt, timestamp);

        // Update control variables
        self.update_pacing_rate();
        self.set_send_cwnd();

        // Update round count
        self.round_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get send cwnd
    pub fn get_send_cwnd(&self) -> u32 {
        self.send_cwnd.load(Ordering::Acquire)
    }

    /// Get pacing rate
    pub fn get_pacing_rate(&self) -> u32 {
        self.pacing_rate.load(Ordering::Acquire)
    }
}

/// Unified congestion control operations trait.
/// All congestion control algorithms implement this interface.
pub trait TcpCongestionOps {
    /// Initialize congestion control state
    fn init(&mut self, mss: u32);

    /// Handle new ACK received (returns new cwnd)
    fn on_ack(&mut self, ack: u32, mss: u32) -> u32;

    /// Handle loss detection (returns new cwnd)
    fn on_loss(&mut self) -> u32;

    /// Handle RTT measurement update
    fn on_rtt_update(&mut self, rtt_ms: u32);

    /// Get current congestion window
    fn cwnd(&self) -> u32;

    /// Get slow start threshold
    fn ssthresh(&self) -> u32;

    /// Get current RTO in ms
    fn rto(&self) -> u32;

    /// Get congestion control algorithm name
    fn name(&self) -> &'static str;
}

impl TcpCongestionOps for RenoCongestionControl {
    fn init(&mut self, mss: u32) {
        self.cwnd.store(mss, Ordering::Release);
        self.ssthresh.store(65535, Ordering::Release);
        self.state.store(CcState::SlowStart as u32, Ordering::Release);
        self.dup_ack_count.store(0, Ordering::Release);
    }

    fn on_ack(&mut self, ack: u32, mss: u32) -> u32 {
        self.on_new_ack(ack, mss);
        self.cwnd.load(Ordering::Acquire)
    }

    fn on_loss(&mut self) -> u32 {
        self.on_timeout();
        self.cwnd.load(Ordering::Acquire)
    }

    fn on_rtt_update(&mut self, rtt_ms: u32) {
        self.update_rtt(rtt_ms);
    }

    fn cwnd(&self) -> u32 {
        self.cwnd.load(Ordering::Acquire)
    }

    fn ssthresh(&self) -> u32 {
        self.ssthresh.load(Ordering::Acquire)
    }

    fn rto(&self) -> u32 {
        self.rto.load(Ordering::Acquire)
    }

    fn name(&self) -> &'static str {
        "reno"
    }
}

impl TcpCongestionOps for CubicCongestionControl {
    fn init(&mut self, mss: u32) {
        self.cwnd.store(mss, Ordering::Release);
        self.ssthresh.store(65535, Ordering::Release);
        self.epoch_start.store(0, Ordering::Release);
        self.w_max.store(0, Ordering::Release);
    }

    fn on_ack(&mut self, _ack: u32, _mss: u32) -> u32 {
        // CUBIC uses timestamp-based window calculation
        // Caller should use on_new_ack(timestamp) instead
        self.cwnd.load(Ordering::Acquire)
    }

    fn on_loss(&mut self) -> u32 {
        self.on_loss();
        self.cwnd.load(Ordering::Acquire)
    }

    fn on_rtt_update(&mut self, _rtt_ms: u32) {
        // CUBIC does not use RTT for window calculation directly
    }

    fn cwnd(&self) -> u32 {
        self.cwnd.load(Ordering::Acquire)
    }

    fn ssthresh(&self) -> u32 {
        self.ssthresh.load(Ordering::Acquire)
    }

    fn rto(&self) -> u32 {
        // CUBIC uses same RTO as Reno by default
        1000
    }

    fn name(&self) -> &'static str {
        "cubic"
    }
}

impl TcpCongestionOps for BbrCongestionControl {
    fn init(&mut self, _mss: u32) {
        self.mode.store(BbrMode::Startup as u32, Ordering::Release);
        self.bw.store(0, Ordering::Release);
        self.min_rtt.store(u32::MAX, Ordering::Release);
    }

    fn on_ack(&mut self, _ack: u32, _mss: u32) -> u32 {
        // BBR uses delivered/interval model
        // Caller should use on_ack(delivered, rtt, timestamp) instead
        self.send_cwnd.load(Ordering::Acquire)
    }

    fn on_loss(&mut self) -> u32 {
        // BBR does not reduce cwnd on loss
        self.send_cwnd.load(Ordering::Acquire)
    }

    fn on_rtt_update(&mut self, rtt_ms: u32) {
        self.update_min_rtt(rtt_ms, 0);
    }

    fn cwnd(&self) -> u32 {
        self.send_cwnd.load(Ordering::Acquire)
    }

    fn ssthresh(&self) -> u32 {
        // BBR does not use ssthresh
        u32::MAX
    }

    fn rto(&self) -> u32 {
        // BBR uses 2 * min_rtt as probe timeout
        let min_rtt = self.min_rtt.load(Ordering::Acquire);
        if min_rtt == u32::MAX { 1000 } else { min_rtt * 2 }
    }

    fn name(&self) -> &'static str {
        "bbr"
    }
}

/// Congestion control algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CongestionAlgo {
    /// Legacy TCP congestion control
    Reno = 0,
    /// Standard TCP default, RFC 8312
    Cubic = 1,
    /// Model-based congestion control
    Bbr = 2,
}

/// Congestion control factory
pub struct CongestionControlFactory;

impl CongestionControlFactory {
    /// Create Reno
    pub fn create_reno() -> RenoCongestionControl {
        RenoCongestionControl::new()
    }

    /// Create CUBIC
    pub fn create_cubic() -> CubicCongestionControl {
        CubicCongestionControl::new()
    }

    /// Create BBR
    pub fn create_bbr() -> BbrCongestionControl {
        BbrCongestionControl::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reno_slow_start() {
        let reno = RenoCongestionControl::new();

        assert_eq!(reno.get_state(), CcState::SlowStart);
        assert_eq!(reno.get_cwnd(), 1460);

        // New ACK in slow start
        reno.on_new_ack(1000, 1460);
        assert_eq!(reno.get_cwnd(), 2920); // Doubled

        reno.on_new_ack(2000, 1460);
        assert_eq!(reno.get_cwnd(), 4380);
    }

    #[test]
    fn test_reno_fast_retransmit() {
        let reno = RenoCongestionControl::new();

        // Set large cwnd
        reno.cwnd.store(10000, Ordering::Release);

        // 3 duplicate ACKs
        reno.on_dup_ack(1000, 1460);
        reno.on_dup_ack(1000, 1460);
        reno.on_dup_ack(1000, 1460);

        assert_eq!(reno.get_state(), CcState::FastRecovery);
        assert_eq!(reno.ssthresh.load(Ordering::Relaxed), 5000);
    }

    #[test]
    fn test_reno_timeout() {
        let reno = RenoCongestionControl::new();

        reno.cwnd.store(10000, Ordering::Release);

        reno.on_timeout();

        assert_eq!(reno.get_state(), CcState::SlowStart);
        assert_eq!(reno.get_cwnd(), 1460);
        assert_eq!(reno.ssthresh.load(Ordering::Relaxed), 5000);
    }

    #[test]
    fn test_reno_rtt_estimation() {
        let reno = RenoCongestionControl::new();

        // First RTT measurement
        reno.update_rtt(100);
        assert_eq!(reno.srtt.load(Ordering::Relaxed), 100);

        // Second measurement
        reno.update_rtt(120);
        assert!(reno.srtt.load(Ordering::Relaxed) > 100);
    }

    #[test]
    fn test_cubic_new() {
        let cubic = CubicCongestionControl::new();

        assert_eq!(cubic.get_cwnd(), 1460);
    }

    #[test]
    fn test_cubic_loss() {
        let cubic = CubicCongestionControl::new();

        cubic.cwnd.store(10000, Ordering::Release);
        cubic.on_loss();

        // W_max should be old cwnd
        assert_eq!(cubic.w_max.load(Ordering::Relaxed), 10000);

        // ssthresh should be ~70% of cwnd
        let ssthresh = cubic.ssthresh.load(Ordering::Relaxed);
        assert!(ssthresh > 6000 && ssthresh < 8000);
    }

    #[test]
    fn test_bbr_new() {
        let bbr = BbrCongestionControl::new();

        assert_eq!(bbr.get_mode(), BbrMode::Startup);
    }

    #[test]
    fn test_bbr_bw_estimation() {
        let bbr = BbrCongestionControl::new();

        // Deliver 1460 bytes in 10ms
        bbr.update_bw(1460, 10000);

        let bw = bbr.bw.load(Ordering::Relaxed);
        assert!(bw > 0);
    }

    #[test]
    fn test_bbr_min_rtt() {
        let bbr = BbrCongestionControl::new();

        bbr.update_min_rtt(100, 0);
        assert_eq!(bbr.min_rtt.load(Ordering::Relaxed), 100);

        bbr.update_min_rtt(50, 1);
        assert_eq!(bbr.min_rtt.load(Ordering::Relaxed), 50);

        bbr.update_min_rtt(200, 2);
        assert_eq!(bbr.min_rtt.load(Ordering::Relaxed), 50);
    }

    #[test]
    fn test_bbr_bdp() {
        let bbr = BbrCongestionControl::new();

        // Set BW = 1 MB/s, RTT = 100ms
        bbr.bw.store(1_000_000, Ordering::Release);
        bbr.min_rtt.store(100_000, Ordering::Release);

        let bdp = bbr.calculate_bdp();
        assert_eq!(bdp, 100_000); // 1 MB/s * 100ms = 100KB
    }
}
