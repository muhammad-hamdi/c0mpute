//! Workload runners — the worker-side counterpart to `dispatch::run_worker_subscriber`.
//!
//! For each enabled role, the supervisor spawns:
//!
//!   1. The dispatch task (subscribes to `c0mpute/jobs/<workload>`,
//!      bids on offers we're capable of, watches for accepts naming us,
//!      and forwards accepted jobs to the runner channel).
//!   2. A runner task (this module) — receives accepted jobs, executes
//!      the workload via the in-process workload code (e.g.
//!      `c0mpute_transcode::transcode`), and publishes a `JobReceipt`.
//!
//! The receipt is published on the same job topic the offer was on, so
//! the buyer (which is still subscribed) sees it.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use c0mpute_net::Libp2pNetwork;
use c0mpute_net::topics::{JobAccept, JobReceipt, JobStatus, job_topic};
use c0mpute_proto::TranscodeSpec;
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Job handed off from the dispatch loop to a per-workload runner.
#[derive(Clone, Debug)]
pub struct AcceptedJob {
    pub accept: JobAccept,
}

/// Phase-1 inline schema for an `ffmpeg.transcode` workload. Lives in
/// `JobOffer.spec_inline` (and again in `JobAccept.spec_inline`).
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct TranscodeJobInline {
    /// HTTP URL the worker should download the input from.
    pub input_url: String,
    /// Human-readable preset (informational only — the actual ffmpeg
    /// args are derived from `spec`).
    pub preset: String,
    /// Concrete TranscodeSpec the worker passes to ffmpeg.
    pub spec: TranscodeSpec,
}

/// Spawn the transcode runner. Returns the sender; clone + give to the
/// dispatch loop so it can deliver `AcceptedJob`s.
pub fn spawn_transcode_runner(
    net: Arc<Libp2pNetwork>,
    cache_root: PathBuf,
    ffmpeg_bin: PathBuf,
) -> mpsc::Sender<AcceptedJob> {
    let (tx, mut rx) = mpsc::channel::<AcceptedJob>(16);
    tokio::spawn(async move {
        let caps_result = c0mpute_transcode::probe_capabilities(&ffmpeg_bin).await;
        let caps = match caps_result {
            Ok(c) => c,
            Err(e) => {
                warn!(err = %e, "transcode runner: ffmpeg probe failed; runner exiting");
                return;
            }
        };
        info!(?caps.encoders, "transcode runner ready");
        while let Some(job) = rx.recv().await {
            let job_id = job.accept.job_id.clone();
            let buyer_peer_id = job.accept.buyer_peer_id.clone();
            match run_transcode_job(&net, &cache_root, &ffmpeg_bin, &caps, &job).await {
                Ok((output_path, output_hash)) => {
                    info!(
                        %job_id,
                        output = %output_path.display(),
                        hash = %output_hash,
                        "transcode runner: job complete"
                    );
                    publish_receipt(
                        &net,
                        &job.accept,
                        Some(output_hash),
                        JobStatus::Completed,
                    )
                    .await;
                }
                Err(e) => {
                    warn!(%job_id, %buyer_peer_id, err = %e, "transcode runner: job failed");
                    publish_receipt(&net, &job.accept, None, JobStatus::Failed).await;
                }
            }
        }
    });
    tx
}

async fn run_transcode_job(
    _net: &Libp2pNetwork,
    cache_root: &std::path::Path,
    ffmpeg_bin: &std::path::Path,
    caps: &c0mpute_transcode::Capabilities,
    job: &AcceptedJob,
) -> Result<(PathBuf, String)> {
    let spec_inline = job
        .accept
        .spec_inline
        .as_ref()
        .context("accept missing spec_inline (Phase 1 requires inline)")?;
    let inline: TranscodeJobInline = serde_json::from_value(spec_inline.clone())
        .context("decode spec_inline as TranscodeJobInline")?;

    let job_dir = cache_root.join("jobs").join(&job.accept.job_id);
    fs::create_dir_all(&job_dir).await?;
    let input_path = job_dir.join("input.bin");
    let output_path = job_dir.join("output.mp4");

    download_to_file(&inline.input_url, &input_path).await?;
    let input_bytes = fs::metadata(&input_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    info!(bytes = input_bytes, path = %input_path.display(), "input downloaded");

    let budget = Duration::from_secs(15 * 60);
    c0mpute_transcode::transcode(
        ffmpeg_bin,
        &input_path,
        &output_path,
        &inline.spec,
        caps,
        budget,
    )
    .await
    .context("ffmpeg")?;

    let output_bytes = fs::read(&output_path).await?;
    let hash = blake3::hash(&output_bytes);
    Ok((output_path, hex::encode(hash.as_bytes())))
}

async fn download_to_file(url: &str, path: &std::path::Path) -> Result<()> {
    let resp = reqwest::get(url).await.context("GET input_url")?;
    if !resp.status().is_success() {
        bail!("input fetch returned {}", resp.status());
    }
    let bytes = resp.bytes().await?;
    fs::write(path, &bytes).await?;
    Ok(())
}

async fn publish_receipt(
    net: &Libp2pNetwork,
    accept: &JobAccept,
    output_hash: Option<String>,
    status: JobStatus,
) {
    let receipt = JobReceipt {
        job_id: accept.job_id.clone(),
        worker_peer_id: net.peer_id().to_base58(),
        worker_did: None,
        buyer_peer_id: accept.buyer_peer_id.clone(),
        output_hash,
        status,
        completed_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        signature: None,
    };
    let topic = job_topic(&accept.workload_type);
    if let Ok(payload) = serde_json::to_vec(&receipt) {
        if let Err(e) = net.publish(&topic, payload).await {
            warn!(err = %e, "publish receipt");
        }
    }
}
