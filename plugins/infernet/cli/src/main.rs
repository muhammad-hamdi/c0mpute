//! `infernet` — AI inference workload CLI for the c0mpute network.
//!
//! In c0mpute v1 (per docs/c0mpute-v1.md and DIP-0005), Infernet Protocol is
//! the inference workload module. The Rust binary here is a thin
//! command-stub aligned with the PRD example surface; the production
//! version delegates to the upstream Infernet runtime when one is
//! installed locally, otherwise submits the job through c0mpute's
//! coordinator like any other workload.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "infernet",
    version,
    about = "infernet — AI inference workload CLI"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run inference for a JSONL prompt file.
    Run {
        prompts: PathBuf,
        #[arg(long)]
        model: String,
        #[arg(long, default_value = "c0mpute")]
        network: String,
        #[arg(long)]
        max_price: Option<f64>,
    },
    /// Diagnostic checks for the local infernet setup.
    Doctor,
    /// Manage / list models known to infernet.
    Models {
        #[command(subcommand)]
        cmd: ModelsCmd,
    },
    /// Run a benchmark against a model.
    Benchmark {
        #[arg(long)]
        model: String,
    },
    /// Print the binary version.
    Version,
}

#[derive(Subcommand, Debug)]
enum ModelsCmd {
    /// List known models.
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Version => println!("infernet {}", env!("CARGO_PKG_VERSION")),
        Cmd::Doctor => {
            println!("OK   infernet binary");
            println!("WARN runtime — not yet wired up to a backing inference engine");
            println!("WARN c0mpute coordinator reachability — not implemented");
        }
        Cmd::Run {
            prompts,
            model,
            network,
            max_price,
        } => {
            println!(
                "[stub] would submit inference job: prompts={} model={} network={} max_price={:?}",
                prompts.display(),
                model,
                network,
                max_price
            );
        }
        Cmd::Models { cmd: ModelsCmd::List } => {
            println!("(no models registered yet — stub)");
        }
        Cmd::Benchmark { model } => {
            println!("[stub] benchmark {model}");
        }
    }
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,infernet=debug"));
    fmt().with_env_filter(filter).try_init().ok();
}
