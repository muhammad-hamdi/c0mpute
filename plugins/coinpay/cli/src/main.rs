//! `coinpay` — DID, wallet, escrow, payments, receipts, and reputation.
//!
//! Per DIP-0007, this is the canonical identity + payment layer for the
//! c0mpute network. Today this binary is a command-stub skeleton; the real
//! implementation will:
//!
//! - Generate / store / rotate identity keys (`coinpay did create`)
//! - Produce DID strings (`did:coinpay:<role>:<id>`)
//! - Sign request envelopes for c0mpute and infernet to use via subprocess
//! - Manage escrow (create, fund, release) against a backing ledger
//! - Track reputation badges per DID

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "coinpay",
    version,
    about = "coinpay — DID, wallet, escrow, payments, receipts, reputation"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Manage decentralized identifiers.
    Did {
        #[command(subcommand)]
        cmd: DidCmd,
    },
    /// Manage wallets linked to a DID.
    Wallet {
        #[command(subcommand)]
        cmd: WalletCmd,
    },
    /// Escrow operations.
    Escrow {
        #[command(subcommand)]
        cmd: EscrowCmd,
    },
    /// List signed receipts.
    Receipts {
        #[command(subcommand)]
        cmd: ReceiptsCmd,
    },
    /// Inspect reputation badges for a DID.
    Reputation {
        #[command(subcommand)]
        cmd: ReputationCmd,
    },
    /// Run diagnostics on the local coinpay setup.
    Doctor,
    /// Print the binary version.
    Version,
}

#[derive(Subcommand, Debug)]
enum DidCmd {
    /// Generate a new DID and identity key.
    Create {
        #[arg(long, default_value = "user")]
        role: String,
    },
    /// Print the active DID and key fingerprint.
    Status,
    /// Export the identity bundle (encrypted).
    Export,
    /// Import an identity bundle.
    Import { path: std::path::PathBuf },
}

#[derive(Subcommand, Debug)]
enum WalletCmd {
    /// Show wallet status (linked addresses, balances).
    Status,
    /// Link an external wallet address to the active DID.
    Link { address: String },
}

#[derive(Subcommand, Debug)]
enum EscrowCmd {
    /// Show open escrows for the active DID.
    Status,
    /// Create + fund an escrow for a job.
    Create {
        #[arg(long)]
        job_id: String,
        #[arg(long)]
        amount: f64,
        #[arg(long, default_value = "USDC")]
        currency: String,
    },
    /// Release escrow after validation.
    Release {
        #[arg(long)]
        escrow_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum ReceiptsCmd {
    /// List recent receipts.
    List {
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
}

#[derive(Subcommand, Debug)]
enum ReputationCmd {
    /// Inspect a DID's reputation badges.
    Inspect { did: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Version => {
            println!("coinpay {}", env!("CARGO_PKG_VERSION"));
        }
        Cmd::Doctor => {
            println!("OK   coinpay binary");
            println!("WARN identity — `coinpay did create` to set up");
            println!("WARN ledger — not connected (stub)");
        }
        Cmd::Did { cmd } => did(cmd),
        Cmd::Wallet { cmd } => wallet(cmd),
        Cmd::Escrow { cmd } => escrow(cmd),
        Cmd::Receipts { cmd } => receipts(cmd),
        Cmd::Reputation { cmd } => reputation(cmd),
    }
    Ok(())
}

fn did(cmd: DidCmd) {
    match cmd {
        DidCmd::Create { role } => {
            println!("[stub] would generate identity key + register did:coinpay:{role}:<id>");
        }
        DidCmd::Status => println!("[stub] no active DID — run `coinpay did create`"),
        DidCmd::Export => println!("[stub] export identity bundle"),
        DidCmd::Import { path } => println!("[stub] import from {}", path.display()),
    }
}

fn wallet(cmd: WalletCmd) {
    match cmd {
        WalletCmd::Status => println!("[stub] no wallets linked"),
        WalletCmd::Link { address } => println!("[stub] would link {address}"),
    }
}

fn escrow(cmd: EscrowCmd) {
    match cmd {
        EscrowCmd::Status => println!("[stub] no open escrows"),
        EscrowCmd::Create {
            job_id,
            amount,
            currency,
        } => println!("[stub] create escrow job={job_id} amount={amount} {currency}"),
        EscrowCmd::Release { escrow_id } => println!("[stub] release {escrow_id}"),
    }
}

fn receipts(cmd: ReceiptsCmd) {
    match cmd {
        ReceiptsCmd::List { limit } => println!("[stub] last {limit} receipts (none yet)"),
    }
}

fn reputation(cmd: ReputationCmd) {
    match cmd {
        ReputationCmd::Inspect { did } => {
            println!("did: {did}");
            println!("status: unknown — local cache empty (stub)");
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,coinpay=debug"));
    fmt().with_env_filter(filter).try_init().ok();
}
