use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;
use tracing::{debug, warn};

use crate::models::{EpochInfo, VoteAccountStatus, RpcResponse};

static RPC_CLIENT: once_cell::sync::Lazy<Client> = once_cell::sync::Lazy::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client")
});

/// Generic JSON-RPC call. Returns (result_value, latency_ms).
pub async fn rpc_call(
    method: &str,
    params: Option<Value>,
    url: &str,
) -> Result<(Value, u128), reqwest::Error> {
    let body = match params {
        Some(p) => json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": p }),
        None    => json!({ "jsonrpc": "2.0", "id": 1, "method": method }),
    };

    debug!("RPC call: {}", method);
    let start = Instant::now();

    let res = RPC_CLIENT
        .post(url)
        .json(&body)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let latency = start.elapsed().as_millis();
    debug!("RPC {} completed in {}ms", method, latency);

    Ok((res, latency))
}

/// Fetch current epoch information.
pub async fn get_epoch_info(url: &str) -> anyhow::Result<(EpochInfo, u128)> {
    let (res, latency) = rpc_call("getEpochInfo", None, url)
        .await
        .map_err(|e| anyhow::anyhow!("getEpochInfo failed: {}", e))?;

    let epoch_info: EpochInfo = serde_json::from_value(
        res["result"].clone()
    ).map_err(|e| anyhow::anyhow!("Failed to parse EpochInfo: {}", e))?;

    Ok((epoch_info, latency))
}

/// Fetch vote accounts — both current and delinquent.
pub async fn get_vote_accounts(url: &str) -> anyhow::Result<VoteAccountStatus> {
    let (res, _) = rpc_call("getVoteAccounts", None, url)
        .await
        .map_err(|e| anyhow::anyhow!("getVoteAccounts failed: {}", e))?;

    let status: VoteAccountStatus = serde_json::from_value(res["result"].clone())
        .map_err(|e| anyhow::anyhow!("Failed to parse VoteAccountStatus: {}", e))?;

    Ok(status)
}

/// Check node health — returns true if healthy.
pub async fn get_health(url: &str) -> (bool, u128) {
    match rpc_call("getHealth", None, url).await {
        Ok((res, latency)) => {
            let healthy = res["result"].as_str() == Some("ok");
            (healthy, latency)
        }
        Err(e) => {
            warn!("getHealth failed: {}", e);
            (false, 9999)
        }
    }
}

/// Get current slot (network tip).
pub async fn get_slot(url: &str) -> anyhow::Result<u64> {
    let (res, _) = rpc_call("getSlot", None, url)
        .await
        .map_err(|e| anyhow::anyhow!("getSlot failed: {}", e))?;

    res["result"].as_u64()
        .ok_or_else(|| anyhow::anyhow!("getSlot returned non-u64"))
}
