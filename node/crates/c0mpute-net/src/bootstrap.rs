//! Bootstrap-list fetcher.
//!
//! On startup the node fetches https://c0mpute.com/bootstrap.json and
//! merges the result with the hardcoded list. If the fetch fails, fall
//! back to the hardcoded list. If both are empty, the node runs in
//! standalone mode (LAN-only via mDNS, useful for dev/test).

use std::time::Duration;

use anyhow::Result;
use libp2p::Multiaddr;
use serde::Deserialize;
use tracing::{debug, warn};

pub const DEFAULT_BOOTSTRAP_URL: &str = "https://c0mpute.com/bootstrap.json";

#[derive(Clone, Debug, Deserialize)]
pub struct BootstrapFile {
    pub version: u32,
    #[serde(default)]
    pub protocol_id: Option<String>,
    pub peers: Vec<BootstrapPeer>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BootstrapPeer {
    pub id: String,
    pub addrs: Vec<String>,
    #[serde(default)]
    pub operator: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
}

/// Fetch the bootstrap list from `url` and convert peers to multiaddrs
/// embedding their peer-id (`/<addr>/p2p/<id>`). Skips entries that fail
/// to parse rather than aborting.
pub async fn fetch(url: &str) -> Result<Vec<Multiaddr>> {
    let body = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let file: BootstrapFile = serde_json::from_str(&body)?;
    Ok(file
        .peers
        .into_iter()
        .flat_map(|p| {
            let id_part = format!("/p2p/{}", p.id);
            p.addrs
                .into_iter()
                .filter_map(move |a| {
                    let full = format!("{}{}", a, id_part);
                    full.parse::<Multiaddr>()
                        .map_err(|e| {
                            debug!(addr = %full, err = %e, "skipping unparseable bootstrap addr");
                        })
                        .ok()
                })
                .collect::<Vec<_>>()
        })
        .collect())
}

/// Merge a fetched list (best-effort) with hardcoded fallbacks. Failures
/// to fetch don't propagate — they just log a warning and return the
/// fallback list.
pub async fn fetch_or_fallback(url: &str, fallback: Vec<Multiaddr>) -> Vec<Multiaddr> {
    match fetch(url).await {
        Ok(mut remote) => {
            for f in fallback {
                if !remote.contains(&f) {
                    remote.push(f);
                }
            }
            remote
        }
        Err(e) => {
            warn!(err = %e, url, "bootstrap fetch failed; using hardcoded fallback list");
            fallback
        }
    }
}
