# Solana Validator Health Dashboard

**Author:** Marchel Penchev  
**Version:** 0.2.0  
**Language:** Rust  
**License:** MIT

> Open-source monitoring & alerting tool for Solana validators — combining RPC telemetry, validator state analysis, and host system metrics into a single operational dashboard.

---

## Features

| Feature | Status |
|---|---|
| RPC health check | ✅ v0.1 |
| RPC latency monitoring | ✅ v0.1 |
| CPU / RAM / Disk metrics | ✅ v0.1 |
| Health scoring engine | ✅ v0.1 |
| **Delinquency detection** | ✅ v0.2 |
| **Slot lag monitoring** | ✅ v0.2 |
| **Vote account analysis** | ✅ v0.2 |
| **Prometheus /metrics exporter** | ✅ v0.2 |
| **Alert system (CLI + log)** | ✅ v0.2 |
| **Parallel RPC calls (tokio::join)** | ✅ v0.2 |
| Web dashboard | 🗓️ v0.3 |
| Email / Discord / Telegram alerts | 🗓️ v0.3 |
| Historical metrics (SQLite) | 🗓️ v0.4 |
| Prometheus exporter | 🗓️ v0.3 |

---

## Quick Start

```bash
# 1. Clone
git clone https://github.com/mrnobody/solana-validator-health
cd solana-validator-health

# 2. Configure
cp config.toml.example config.toml
# Edit config.toml with your vote account and RPC URL

# 3. Build & run
cargo build --release
./target/release/solana-validator-health
```

---

## Configuration (`config.toml`)

```toml
rpc_url = "https://api.mainnet-beta.solana.com"
validator_identity = "YOUR_VALIDATOR_IDENTITY_PUBKEY"
vote_account = "YOUR_VOTE_ACCOUNT_PUBKEY"

poll_interval_secs = 30

prometheus_enabled = true
prometheus_port = 9090

alert_score_threshold = 70
alert_on_delinquency = true
alert_slot_lag_threshold = 150
```

---

## Health Score

The health score is a composite 0–100 metric:

| Condition | Penalty |
|---|---|
| RPC node unhealthy | -30 |
| RPC latency > 1000ms | -25 |
| RPC latency > 500ms | -15 |
| Validator DELINQUENT | -40 |
| Slot lag > 300 | -20 |
| Slot lag > 150 | -10 |
| CPU > 95% | -20 |
| CPU > 85% | -10 |
| Memory > 28 GB | -15 |
| Disk < 10 GB free | -15 |
| Disk < 50 GB free | -5 |

**Score labels:** EXCELLENT (90–100) · GOOD (75–89) · DEGRADED (60–74) · POOR (40–59) · CRITICAL (0–39)

---

## Prometheus Integration

When `prometheus_enabled = true`, metrics are exposed at:

```
http://localhost:9090/metrics
```

### Available Metrics

```
solana_validator_health_score          # Composite score (0-100)
solana_validator_rpc_latency_ms        # RPC latency
solana_validator_rpc_healthy           # 1=healthy, 0=unhealthy
solana_validator_delinquent            # 1=delinquent, 0=active
solana_validator_slot_lag              # Slots behind tip
solana_validator_last_vote_slot        # Last vote slot
solana_validator_activated_stake_sol   # Stake in SOL
solana_validator_epoch                 # Current epoch
solana_host_cpu_usage_percent          # Host CPU %
solana_host_memory_used_kb             # Host memory KB
solana_host_disk_available_bytes       # Host disk bytes
solana_validator_alerts_total          # Alert counter
```

### Grafana Dashboard

Import the bundled `grafana-dashboard.json` (coming in v0.3) or point your Grafana instance at the Prometheus scrape target.

---

## Roadmap

### v0.3 — Alerting & Web UI
- Email / Discord / Telegram alert delivery
- Embedded web dashboard (axum + htmx)
- Grafana dashboard JSON

### v0.4 — Historical Metrics
- SQLite local time-series storage
- Epoch performance history
- Credit score trend analysis

### v0.5 — Advanced Analytics
- Skip rate analysis
- Epoch reward estimation
- Multi-validator monitoring
- Prometheus Alertmanager integration

---

## Contributing

PRs welcome. Open an issue first for major changes.

---

## License

MIT
