use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use prometheus::{
    register_gauge, register_int_gauge, register_counter,
    Gauge, IntGauge, Counter, Encoder, TextEncoder,
};
use once_cell::sync::Lazy;

// ── System Metrics ────────────────────────────────────────────────────────────

pub struct SystemMetrics {
    pub cpu: f32,
    pub memory_used_kb: u64,
    pub disk_available_bytes: u64,
}

pub fn collect_system() -> SystemMetrics {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.global_cpu_info().cpu_usage();
    let memory_used_kb = sys.used_memory();
    let disk_available_bytes = sys.disks().iter().map(|d| d.available_space()).sum();

    SystemMetrics { cpu, memory_used_kb, disk_available_bytes }
}

// ── Prometheus Registry ───────────────────────────────────────────────────────

pub static PROM_HEALTH_SCORE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "solana_validator_health_score",
        "Composite health score (0-100)"
    ).unwrap()
});

pub static PROM_RPC_LATENCY: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "solana_validator_rpc_latency_ms",
        "RPC call latency in milliseconds"
    ).unwrap()
});

pub static PROM_RPC_HEALTHY: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "solana_validator_rpc_healthy",
        "1 if RPC node reports healthy, 0 otherwise"
    ).unwrap()
});

pub static PROM_DELINQUENT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "solana_validator_delinquent",
        "1 if validator is currently delinquent, 0 if active"
    ).unwrap()
});

pub static PROM_SLOT_LAG: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "solana_validator_slot_lag",
        "Number of slots the validator is behind the network tip"
    ).unwrap()
});

pub static PROM_LAST_VOTE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "solana_validator_last_vote_slot",
        "Slot of the validator's last vote"
    ).unwrap()
});

pub static PROM_ACTIVATED_STAKE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "solana_validator_activated_stake_sol",
        "Total activated stake in SOL"
    ).unwrap()
});

pub static PROM_CPU: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "solana_host_cpu_usage_percent",
        "Host CPU usage percentage"
    ).unwrap()
});

pub static PROM_MEMORY: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "solana_host_memory_used_kb",
        "Host memory used in kilobytes"
    ).unwrap()
});

pub static PROM_DISK: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "solana_host_disk_available_bytes",
        "Host disk available space in bytes"
    ).unwrap()
});

pub static PROM_ALERT_COUNT: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "solana_validator_alerts_total",
        "Total number of alerts fired since startup"
    ).unwrap()
});

pub static PROM_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "solana_validator_epoch",
        "Current Solana epoch"
    ).unwrap()
});

/// Force-initialize all metrics so they appear in /metrics from startup.
pub fn init_metrics() {
    let _ = &*PROM_HEALTH_SCORE;
    let _ = &*PROM_RPC_LATENCY;
    let _ = &*PROM_RPC_HEALTHY;
    let _ = &*PROM_DELINQUENT;
    let _ = &*PROM_SLOT_LAG;
    let _ = &*PROM_LAST_VOTE;
    let _ = &*PROM_ACTIVATED_STAKE;
    let _ = &*PROM_CPU;
    let _ = &*PROM_MEMORY;
    let _ = &*PROM_DISK;
    let _ = &*PROM_ALERT_COUNT;
    let _ = &*PROM_EPOCH;
}

/// Render all Prometheus metrics to text format.
pub fn render_prometheus() -> Vec<u8> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).expect("encode failed");
    buffer
}
