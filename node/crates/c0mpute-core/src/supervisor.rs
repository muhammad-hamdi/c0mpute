//! Long-running supervisor that owns the per-role tasks.

use std::sync::Arc;

use anyhow::Result;
use c0mpute_net::{ChunkSource, Libp2pNetwork, Network, NetworkConfig};
use c0mpute_proto::Hash;
use c0mpute_store::ChunkStore;
use tracing::info;

use crate::{Config, config};

pub struct Supervisor {
    pub config: Config,
    pub store: ChunkStore,
    pub net: Arc<dyn Network>,
}

impl Supervisor {
    pub async fn boot(config: Config) -> Result<Self> {
        std::fs::create_dir_all(&config.storage.root)?;
        let store = ChunkStore::open(&config.storage.root).await?;

        // Real libp2p network. Identity persists at
        // <config_dir>/identity.key. Bootstrap list is empty for now —
        // DIP-0010 wires up c0mpute.com/bootstrap.json once we run
        // public seed nodes.
        let identity_dir = config::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let local_source: Arc<dyn ChunkSource> =
            Arc::new(StoreSource(store.clone()));

        let net_cfg = NetworkConfig::for_dir(identity_dir)
            .with_local_source(local_source);

        let libp2p_net = Libp2pNetwork::spawn(net_cfg).await?;
        info!(
            peer_id = %libp2p_net.peer_id(),
            "libp2p network up"
        );
        let net: Arc<dyn Network> = Arc::new(libp2p_net);

        Ok(Self {
            config,
            store,
            net,
        })
    }

    pub async fn run(self) -> Result<()> {
        info!(
            roles = ?self.config.roles,
            store = %self.config.storage.root.display(),
            "supervisor up"
        );

        if self.config.roles.contains(&c0mpute_proto::Role::Gateway) {
            let bind: std::net::SocketAddr = self.config.gateway.bind.parse()?;
            let state = c0mpute_gateway::GatewayState {
                store: self.store.clone(),
                net: self.net.clone(),
            };
            tokio::spawn(async move {
                if let Err(e) = c0mpute_gateway::serve(state, bind).await {
                    tracing::error!(err = %e, "gateway server exited");
                }
            });
        }

        if self.config.update_auto {
            let feed = self
                .config
                .update_feed_url
                .clone()
                .unwrap_or_else(|| c0mpute_update::DEFAULT_RELEASE_FEED.to_string());
            let interval =
                std::time::Duration::from_secs(self.config.update_interval_secs.max(60));
            let current = env!("CARGO_PKG_VERSION").to_string();
            info!(
                interval_secs = interval.as_secs(),
                feed = %feed,
                "auto-upgrade poller starting"
            );
            tokio::spawn(c0mpute_update::poll_loop(current, feed, interval));
        }

        // Hold open until ctrl-c.
        tokio::signal::ctrl_c().await?;
        info!("ctrl-c received; shutting down");
        Ok(())
    }
}

struct StoreSource(ChunkStore);

#[async_trait::async_trait]
impl ChunkSource for StoreSource {
    async fn read_chunk(&self, hash: &Hash) -> Result<Vec<u8>> {
        self.0.get(hash).await
    }
}
