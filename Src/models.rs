use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ── RPC Response Wrapper ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

// ── Solana RPC Types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct EpochInfo {
    pub epoch: u64,
    #[serde(rename = "slotIndex")]
    pub slot_index: u64,
    #[serde(rename = "slotsInEpoch")]
    pub slots_in_epoch: u64,
    #[serde(rename = "absoluteSlot")]
    pub absolute_slot: u64,
    #[serde(rename = "blockHeight")]
    pub block_height: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VoteAccountStatus {
    pub current: Vec<VoteAccount>,
    pub delinquent: Vec<VoteAccount>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VoteAccount {
    #[serde(rename = "votePubkey")]
    pub vote_pubkey: String,
    #[serde(rename = "nodePubkey")]
    pub node_pubkey: String,
    #[serde(rename = "activatedStake")]
    pub activated_stake: u64,
    #[serde(rename = "epochVoteAccount")]
    pub epoch_vote_account: bool,
    pub commission: u8,
    #[serde(rename = "lastVote")]
    pub last_vote: u64,
    #[serde(rename = "epochCredits")]
    pub epoch_credits: Vec<(u64, u64, u64)>,
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ValidatorSnapshot {
    pub timestamp: DateTime<Utc>,
    pub epoch: u64,
    pub slot: u64,
    pub slot_lag: i64,
    pub is_delinquent: bool,
    pub last_vote: u64,
    pub activated_stake: u64,
    pub commission: u8,
    pub rpc_latency_ms: u128,
    pub rpc_healthy: bool,
    pub cpu_usage: f32,
    pub memory_used_kb: u64,
    pub disk_available_bytes: u64,
    pub health_score: u32,
    pub alerts: Vec<String>,
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub validator_identity: String,
    pub vote_account: String,
    #[serde(default = "default_poll")]
    pub poll_interval_secs: u64,
    #[serde(default)]
    pub prometheus_enabled: bool,
    #[serde(default = "default_prom_port")]
    pub prometheus_port: u16,
    #[serde(default = "default_score_threshold")]
    pub alert_score_threshold: u32,
    #[serde(default = "default_true")]
    pub alert_on_delinquency: bool,
    #[serde(default = "default_slot_lag")]
    pub alert_slot_lag_threshold: i64,
}

fn default_poll() -> u64 { 30 }
fn default_prom_port() -> u16 { 9090 }
fn default_score_threshold() -> u32 { 70 }
fn default_true() -> bool { true }
fn default_slot_lag() -> i64 { 150 }
