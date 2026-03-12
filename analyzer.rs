use crate::metrics::SystemMetrics;
use crate::models::{Config, ValidatorSnapshot, VoteAccountStatus};

// ── Delinquency Analysis ─────────────────────────────────────────────────────

pub struct ValidatorState {
    pub is_delinquent: bool,
    pub last_vote: u64,
    pub activated_stake: u64,
    pub commission: u8,
    pub slot_lag: i64,
}

pub fn analyze_validator(
    vote_accounts: &VoteAccountStatus,
    vote_pubkey: &str,
    current_slot: u64,
) -> ValidatorState {
    // Check delinquent list first
    if let Some(va) = vote_accounts.delinquent.iter().find(|v| v.vote_pubkey == vote_pubkey) {
        let slot_lag = current_slot as i64 - va.last_vote as i64;
        return ValidatorState {
            is_delinquent: true,
            last_vote: va.last_vote,
            activated_stake: va.activated_stake,
            commission: va.commission,
            slot_lag: slot_lag.max(0),
        };
    }

    // Check active list
    if let Some(va) = vote_accounts.current.iter().find(|v| v.vote_pubkey == vote_pubkey) {
        let slot_lag = (current_slot as i64 - va.last_vote as i64).max(0);
        return ValidatorState {
            is_delinquent: false,
            last_vote: va.last_vote,
            activated_stake: va.activated_stake,
            commission: va.commission,
            slot_lag,
        };
    }

    // Vote account not found
    ValidatorState {
        is_delinquent: false,
        last_vote: 0,
        activated_stake: 0,
        commission: 0,
        slot_lag: 0,
    }
}

// ── Health Scoring ────────────────────────────────────────────────────────────

pub fn compute_health_score(
    rpc_latency: u128,
    rpc_healthy: bool,
    validator: &ValidatorState,
    sys: &SystemMetrics,
) -> u32 {
    let mut score: i32 = 100;

    // RPC health
    if !rpc_healthy {
        score -= 30;
    } else if rpc_latency > 1000 {
        score -= 25;
    } else if rpc_latency > 500 {
        score -= 15;
    } else if rpc_latency > 250 {
        score -= 5;
    }

    // Delinquency — hard penalty
    if validator.is_delinquent {
        score -= 40;
    }

    // Slot lag
    if validator.slot_lag > 300 {
        score -= 20;
    } else if validator.slot_lag > 150 {
        score -= 10;
    } else if validator.slot_lag > 50 {
        score -= 5;
    }

    // CPU pressure
    if sys.cpu > 95.0 {
        score -= 20;
    } else if sys.cpu > 85.0 {
        score -= 10;
    } else if sys.cpu > 75.0 {
        score -= 5;
    }

    // Memory (in KB — 16 GB = 16_000_000 KB)
    if sys.memory_used_kb > 28_000_000 {
        score -= 15;
    } else if sys.memory_used_kb > 20_000_000 {
        score -= 8;
    }

    // Disk (warn if < 50 GB free)
    if sys.disk_available_bytes < 10_000_000_000 {
        score -= 15;
    } else if sys.disk_available_bytes < 50_000_000_000 {
        score -= 5;
    }

    score.max(0) as u32
}

// ── Alert Generation ──────────────────────────────────────────────────────────

pub fn evaluate_alerts(snapshot: &ValidatorSnapshot, config: &Config) -> Vec<String> {
    let mut alerts = Vec::new();

    if snapshot.health_score < config.alert_score_threshold {
        alerts.push(format!(
            "⚠️  Health score {} is below threshold {}",
            snapshot.health_score, config.alert_score_threshold
        ));
    }

    if config.alert_on_delinquency && snapshot.is_delinquent {
        alerts.push("🚨 DELINQUENT: Validator is not voting!".to_string());
    }

    if snapshot.slot_lag > config.alert_slot_lag_threshold {
        alerts.push(format!(
            "⏱️  Slot lag {} exceeds threshold {}",
            snapshot.slot_lag, config.alert_slot_lag_threshold
        ));
    }

    if !snapshot.rpc_healthy {
        alerts.push("❌ RPC node is reporting unhealthy".to_string());
    }

    if snapshot.rpc_latency_ms > 1000 {
        alerts.push(format!(
            "🐢 High RPC latency: {}ms", snapshot.rpc_latency_ms
        ));
    }

    if snapshot.cpu_usage > 90.0 {
        alerts.push(format!(
            "🔥 Critical CPU usage: {:.1}%", snapshot.cpu_usage
        ));
    }

    if snapshot.disk_available_bytes < 10_000_000_000 {
        alerts.push(format!(
            "💾 Critical: Only {:.1} GB disk remaining",
            snapshot.disk_available_bytes as f64 / 1_000_000_000.0
        ));
    }

    alerts
}

// ── Score Label ───────────────────────────────────────────────────────────────

pub fn score_label(score: u32) -> &'static str {
    match score {
        90..=100 => "EXCELLENT",
        75..=89  => "GOOD",
        60..=74  => "DEGRADED",
        40..=59  => "POOR",
        _        => "CRITICAL",
    }
}
