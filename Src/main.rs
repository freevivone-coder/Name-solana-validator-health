mod rpc;
mod metrics;
mod analyzer;
mod models;
mod prometheus_server;
mod display;

use std::fs;
use chrono::Utc;
use tracing::{info, warn, error};
use tracing_subscriber::EnvFilter;

use rpc::{get_epoch_info, get_vote_accounts, get_health, get_slot};
use metrics::{
    collect_system, init_metrics,
    PROM_HEALTH_SCORE, PROM_RPC_LATENCY, PROM_RPC_HEALTHY,
    PROM_DELINQUENT, PROM_SLOT_LAG, PROM_LAST_VOTE,
    PROM_ACTIVATED_STAKE, PROM_CPU, PROM_MEMORY, PROM_DISK,
    PROM_ALERT_COUNT, PROM_EPOCH,
};
use analyzer::{analyze_validator, compute_health_score, evaluate_alerts};
use display::render_dashboard;
use models::{Config, ValidatorSnapshot};

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("solana_validator_health=info".parse().unwrap()))
        .init();

    println!("╔══════════════════════════════════════════════╗");
    println!("║   Solana Validator Health Dashboard v0.2.0   ║");
    println!("║   Author: Marchel Penchev                          ║");
    println!("╚══════════════════════════════════════════════╝");

    // Load config
    let config_str = fs::read_to_string("config.toml")
        .expect("config.toml not found in working directory");
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse config.toml");

    info!("RPC endpoint: {}", config.rpc_url);
    info!("Vote account: {}", config.vote_account);
    info!("Polling every {} seconds", config.poll_interval_secs);

    // Init Prometheus metrics
    init_metrics();

    // Spawn Prometheus HTTP server if enabled
    if config.prometheus_enabled {
        let port = config.prometheus_port;
        tokio::spawn(async move {
            prometheus_server::start_prometheus_server(port).await;
        });
        info!("Prometheus exporter started on :{}/metrics", config.prometheus_port);
    }

    // Main monitoring loop
    let interval = tokio::time::Duration::from_secs(config.poll_interval_secs);
    loop {
        match collect_snapshot(&config).await {
            Ok(snapshot) => {
                // Update Prometheus gauges
                PROM_HEALTH_SCORE.set(snapshot.health_score as i64);
                PROM_RPC_LATENCY.set(snapshot.rpc_latency_ms as f64);
                PROM_RPC_HEALTHY.set(if snapshot.rpc_healthy { 1 } else { 0 });
                PROM_DELINQUENT.set(if snapshot.is_delinquent { 1 } else { 0 });
                PROM_SLOT_LAG.set(snapshot.slot_lag);
                PROM_LAST_VOTE.set(snapshot.last_vote as i64);
                PROM_ACTIVATED_STAKE.set(snapshot.activated_stake as f64 / 1_000_000_000.0);
                PROM_CPU.set(snapshot.cpu_usage as f64);
                PROM_MEMORY.set(snapshot.memory_used_kb as f64);
                PROM_DISK.set(snapshot.disk_available_bytes as f64);
                PROM_EPOCH.set(snapshot.epoch as i64);

                if !snapshot.alerts.is_empty() {
                    PROM_ALERT_COUNT.inc_by(snapshot.alerts.len() as f64);
                }

                // Render CLI dashboard
                render_dashboard(&snapshot);

                // Log alerts
                for alert in &snapshot.alerts {
                    warn!("{}", alert);
                }
            }
            Err(e) => {
                error!("Failed to collect snapshot: {}", e);
            }
        }

        tokio::time::sleep(interval).await;
    }
}

async fn collect_snapshot(config: &Config) -> anyhow::Result<ValidatorSnapshot> {
    // Parallel RPC calls
    let epoch_fut = get_epoch_info(&config.rpc_url);
    let vote_fut  = get_vote_accounts(&config.rpc_url);
    let health_fut = get_health(&config.rpc_url);
    let slot_fut  = get_slot(&config.rpc_url);

    let (epoch_res, vote_res, (rpc_healthy, rpc_latency), slot_res) =
        tokio::join!(epoch_fut, vote_fut, health_fut, slot_fut);

    let (epoch_info, _epoch_latency) = epoch_res?;
    let vote_accounts = vote_res?;
    let current_slot  = slot_res.unwrap_or(epoch_info.absolute_slot);

    // Validator state analysis
    let validator = analyze_validator(&vote_accounts, &config.vote_account, current_slot);

    // System metrics
    let sys = collect_system();

    // Health score
    let health_score = compute_health_score(rpc_latency, rpc_healthy, &validator, &sys);

    let mut snapshot = ValidatorSnapshot {
        timestamp: Utc::now(),
        epoch: epoch_info.epoch,
        slot: current_slot,
        slot_lag: validator.slot_lag,
        is_delinquent: validator.is_delinquent,
        last_vote: validator.last_vote,
        activated_stake: validator.activated_stake,
        commission: validator.commission,
        rpc_latency_ms: rpc_latency,
        rpc_healthy,
        cpu_usage: sys.cpu,
        memory_used_kb: sys.memory_used_kb,
        disk_available_bytes: sys.disk_available_bytes,
        health_score,
        alerts: vec![],
    };

    // Evaluate alerts
    snapshot.alerts = evaluate_alerts(&snapshot, config);

    Ok(snapshot)
}
