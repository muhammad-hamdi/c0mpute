//! High-level erasure-coded storage on top of `ChunkStore`.
//!
//! `Storage::put(bytes)` →
//!   1. Hash the plaintext (object_hash = blake3(plaintext)).
//!   2. RS-encode into 14 shards (k=10, parity=4).
//!   3. Write each shard into the underlying ChunkStore (keyed by its
//!      own blake3 hash).
//!   4. Persist a manifest at `manifests/<object_hash>.json` mapping
//!      object → shard hashes + indices.
//!
//! `Storage::get(object_hash)` →
//!   1. Load the manifest.
//!   2. Read each shard from the chunk store; missing shards are
//!      tolerated up to the parity budget.
//!   3. RS-decode and return.
//!
//! Single-node today (Phase 1 of DIP-0012). Phase 2 distributes
//! shards across peers; the `[ShardEntry::host_hint]` field already
//! exists for that.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use c0mpute_proto::Hash;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, warn};

use crate::ChunkStore;
use crate::erasure::{self, DEFAULT_K, DEFAULT_PARITY, Shard};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShardEntry {
    pub index: u8,
    pub hash: Hash,
    /// Host hint for cross-node placement (peer id). `None` = local.
    /// Populated by Phase 2 placement; ignored by single-node mode.
    pub host_hint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectManifest {
    pub object_hash: Hash,
    pub original_len: u64,
    pub k: u8,
    pub parity: u8,
    pub shards: Vec<ShardEntry>,
}

#[derive(Clone, Debug)]
pub struct Storage {
    inner: ChunkStore,
}

impl Storage {
    pub fn new(inner: ChunkStore) -> Self {
        Self { inner }
    }

    pub fn chunk_store(&self) -> &ChunkStore {
        &self.inner
    }

    fn manifest_path(&self, object_hash: &Hash) -> PathBuf {
        let hex = object_hash.to_hex();
        self.inner
            .root()
            .join("manifests")
            .join(&hex[0..2])
            .join(format!("{}.json", hex))
    }

    /// Store an object. Returns the object's blake3 hash + the manifest.
    pub async fn put(&self, data: &[u8]) -> Result<ObjectManifest> {
        let object_hash = Hash::of(data);
        let (shards, original_len) = erasure::encode(data, DEFAULT_K, DEFAULT_PARITY)?;

        let mut entries = Vec::with_capacity(shards.len());
        for s in &shards {
            // Write each shard into the underlying chunk store, keyed
            // by its own hash. The chunk store handles atomic writes.
            let h = self.inner.put(&s.bytes).await?;
            entries.push(ShardEntry {
                index: s.index,
                hash: h,
                host_hint: None,
            });
        }

        let manifest = ObjectManifest {
            object_hash,
            original_len: original_len as u64,
            k: DEFAULT_K as u8,
            parity: DEFAULT_PARITY as u8,
            shards: entries,
        };

        self.write_manifest(&manifest).await?;
        debug!(
            object_hash = %object_hash,
            shard_count = manifest.shards.len(),
            "stored object"
        );
        Ok(manifest)
    }

    /// Read an object back. Tolerates up to `parity` missing shards.
    pub async fn get(&self, object_hash: &Hash) -> Result<Vec<u8>> {
        let manifest = self.read_manifest(object_hash).await?;
        let n = (manifest.k + manifest.parity) as usize;
        let mut received: Vec<Option<Shard>> = vec![None; n];

        for entry in &manifest.shards {
            match self.inner.get(&entry.hash).await {
                Ok(bytes) => {
                    received[entry.index as usize] = Some(Shard {
                        index: entry.index,
                        bytes,
                    });
                }
                Err(e) => {
                    warn!(
                        object_hash = %object_hash,
                        shard_index = entry.index,
                        err = %e,
                        "shard unreadable; falling back to parity"
                    );
                }
            }
        }

        let plaintext = erasure::decode(
            received,
            manifest.k as usize,
            manifest.parity as usize,
            manifest.original_len as usize,
        )?;

        // Verify integrity end-to-end: the plaintext we just decoded
        // must match the manifest's object_hash.
        let actual = Hash::of(&plaintext);
        if actual != manifest.object_hash {
            anyhow::bail!(
                "object integrity failure: manifest says {} but decoded bytes hash to {}",
                manifest.object_hash,
                actual
            );
        }
        Ok(plaintext)
    }

    pub async fn has(&self, object_hash: &Hash) -> bool {
        fs::metadata(self.manifest_path(object_hash)).await.is_ok()
    }

    /// Delete an object's manifest + every shard it points at.
    pub async fn delete(&self, object_hash: &Hash) -> Result<()> {
        let manifest = match self.read_manifest(object_hash).await {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };
        for entry in &manifest.shards {
            let _ = self.inner.delete(&entry.hash).await;
        }
        let _ = fs::remove_file(self.manifest_path(object_hash)).await;
        Ok(())
    }

    async fn write_manifest(&self, m: &ObjectManifest) -> Result<()> {
        let path = self.manifest_path(&m.object_hash);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_vec_pretty(m).context("serialize manifest")?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, &json).await?;
        fs::rename(&tmp, &path).await?;
        Ok(())
    }

    async fn read_manifest(&self, object_hash: &Hash) -> Result<ObjectManifest> {
        let path = self.manifest_path(object_hash);
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("read manifest {}", path.display()))?;
        let m: ObjectManifest = serde_json::from_slice(&bytes)
            .context("parse manifest JSON")?;
        if m.object_hash != *object_hash {
            return Err(anyhow!(
                "manifest object_hash {} != requested {}",
                m.object_hash,
                object_hash
            ));
        }
        Ok(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> Storage {
        let dir = std::env::temp_dir().join(format!(
            "c0mpute-storage-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let cs = ChunkStore::open(&dir).await.unwrap();
        Storage::new(cs)
    }

    #[tokio::test]
    async fn put_get_roundtrip() {
        let s = store().await;
        let data = b"hello c0mpute erasure-coded storage".repeat(50);
        let m = s.put(&data).await.unwrap();
        assert_eq!(m.shards.len(), 14);
        let out = s.get(&m.object_hash).await.unwrap();
        assert_eq!(out, data);
    }

    /// Pseudo-random heterogeneous bytes. Critical for shard-level
    /// failure tests because identical shard content collides in the
    /// content-addressed chunk store (a real, desirable dedup property
    /// but it makes "lose 4 of 14" meaningless if the 4 share a hash).
    fn varied(len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len);
        let mut state: u64 = 0x9e3779b97f4a7c15;
        for _ in 0..len {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            out.push((state & 0xff) as u8);
        }
        out
    }

    #[tokio::test]
    async fn survives_four_lost_shards() {
        let s = store().await;
        let data = varied(50_000);
        let m = s.put(&data).await.unwrap();
        for entry in m.shards.iter().take(4) {
            s.inner.delete(&entry.hash).await.unwrap();
        }
        let out = s.get(&m.object_hash).await.unwrap();
        assert_eq!(out, data);
    }

    #[tokio::test]
    async fn fails_on_five_lost_shards() {
        let s = store().await;
        let data = varied(20_000);
        let m = s.put(&data).await.unwrap();
        for entry in m.shards.iter().take(5) {
            s.inner.delete(&entry.hash).await.unwrap();
        }
        let err = s.get(&m.object_hash).await.unwrap_err();
        assert!(
            err.to_string().contains("at least") || err.to_string().contains("need"),
            "expected decode-shortage error, got: {err}"
        );
    }
}
