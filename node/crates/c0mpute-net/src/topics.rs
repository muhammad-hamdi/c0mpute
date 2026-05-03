//! Gossipsub topic conventions for c0mpute.
//!
//! Three topic families today:
//!
//!   c0mpute/cap/v1                  — capability advertisements
//!                                     (workers broadcast what they can do)
//!   c0mpute/jobs/<workload-type>    — job dispatch
//!                                     (buyers post jobs; workers claim)
//!   c0mpute/heartbeat/v1            — liveness
//!                                     (every N seconds per worker)
//!
//! Topic identifiers are stable strings; workloads are namespaced by type
//! (e.g. `c0mpute/jobs/ffmpeg.transcode`, `c0mpute/jobs/infernet.inference`).

pub const CAPABILITY_TOPIC: &str = "c0mpute/cap/v1";
pub const HEARTBEAT_TOPIC: &str = "c0mpute/heartbeat/v1";

/// Build the topic name for a workload type.
pub fn job_topic(workload_type: &str) -> String {
    format!("c0mpute/jobs/{workload_type}")
}

/// Capability advertisement payload. Signed by the worker's CoinPay DID
/// once that lands; today we rely on gossipsub message authenticity
/// (libp2p signs each pubsub message with the publisher's identity key).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CapabilityAd {
    /// Worker's libp2p peer-id, hex-encoded.
    pub peer_id: String,
    /// Capability tags this worker advertises, e.g.
    /// `["c0mpute:transcode:h264:nvenc", "c0mpute:gpu:nvidia"]`.
    pub tags: Vec<String>,
    /// Free-disk / free-VRAM / region etc. — opaque JSON for now.
    pub hardware: serde_json::Value,
    /// Unix-ms when this ad was created. Older than ~5min = ignore.
    pub published_at_ms: u64,
}

impl CapabilityAd {
    pub fn now(peer_id: String, tags: Vec<String>, hardware: serde_json::Value) -> Self {
        Self {
            peer_id,
            tags,
            hardware,
            published_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }
}
