use crate::models::ValidatorSnapshot;
use crate::analyzer::score_label;
use chrono::Local;

pub fn render_dashboard(snap: &ValidatorSnapshot) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let label = score_label(snap.health_score);

    println!("\x1b[2J\x1b[1;1H"); // clear screen
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Solana Validator Health Dashboard  │  {}  ║", now);
    println!("╠══════════════════════════════════════════════════════════════╣");

    // Health score with colour
    let score_color = match snap.health_score {
        90..=100 => "\x1b[92m", // bright green
        75..=89  => "\x1b[32m", // green
        60..=74  => "\x1b[33m", // yellow
        40..=59  => "\x1b[31m", // red
        _        => "\x1b[91m", // bright red
    };
    println!(
        "║  Health Score: {}{:3} / 100  [{}]{}\x1b[0m{}",
        score_color,
        snap.health_score,
        label,
        " ".repeat(48usize.saturating_sub(label.len())),
        "║"
    );

    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  VALIDATOR                                                   ║");

    let delinq_str = if snap.is_delinquent {
        "\x1b[91mDELINQUENT\x1b[0m"
    } else {
        "\x1b[92mACTIVE\x1b[0m    "
    };
    println!("║   Status      : {}                                  ║", delinq_str);
    println!("║   Epoch       : {:>10}                                    ║", snap.epoch);
    println!("║   Current Slot: {:>10}                                    ║", snap.slot);
    println!("║   Slot Lag    : {:>10}                                    ║", snap.slot_lag);
    println!("║   Last Vote   : {:>10}                                    ║", snap.last_vote);
    println!(
        "║   Stake       : {:>10.2} SOL                                ║",
        snap.activated_stake as f64 / 1_000_000_000.0
    );
    println!("║   Commission  : {:>9}%                                    ║", snap.commission);

    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  RPC                                                         ║");

    let rpc_status = if snap.rpc_healthy { "\x1b[92mHEALTHY\x1b[0m" } else { "\x1b[91mUNHEALTHY\x1b[0m" };
    println!("║   Status      : {}                                    ║", rpc_status);
    println!("║   Latency     : {:>8} ms                                    ║", snap.rpc_latency_ms);

    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  SYSTEM                                                      ║");
    println!("║   CPU         : {:>7.1} %                                     ║", snap.cpu_usage);
    println!("║   Memory Used : {:>7.1} GB                                    ║", snap.memory_used_kb as f64 / 1_000_000.0);
    println!(
        "║   Disk Free   : {:>7.1} GB                                    ║",
        snap.disk_available_bytes as f64 / 1_000_000_000.0
    );

    if !snap.alerts.is_empty() {
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  ALERTS                                                      ║");
        for alert in &snap.alerts {
            println!("║   {}{}║", alert, " ".repeat(62usize.saturating_sub(alert.len() + 3)));
        }
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("  Author: Marchel Penchev  │  Refresh every {}s  │  Ctrl+C to exit", 30);
}
