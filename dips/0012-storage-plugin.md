---
dip: 0012
title: "Storage as an opt-in plugin: Reed-Solomon 10/14 with HTTP-shaped daemon"
status: Accepted
authors:
  - anthony@profullstack.com
created: 2026-05-03
updated: 2026-05-03
discussion:
implementation: scaffolded in c0mpute-store erasure module + c0mpute-gateway shard endpoints
supersedes:
superseded-by:
---

## Summary

Storage on c0mpute is **opt-in via a plugin role**, not a core network
guarantee. v1 c0mpute jobs default to customer-supplied storage URLs
(S3, R2, B2, anything addressable). Workers that opt into the storage
role accept Reed-Solomon-encoded shards and serve them on demand.

The wire format is **HTTP**, mirroring infernet-protocol's daemon
shape (the c0mpute daemon already serves an axum gateway for chunk
GETs; we extend it for PUT and shard operations). No raw libp2p data
plane for storage in v1 — coordination uses signed HTTP envelopes.

Scheme: **Reed-Solomon 10/14** (10 data + 4 parity = 40% overhead,
durability of ~3 copies at one-third the cost). Auto-repair: when a
shard's host is detected offline, the storage role on a healthy node
generates a replacement shard from the surviving 10 and announces it.

## Motivation

The user's instinct on this:

> you basically need 3 nodes with a copy in case one goes down the
> other two add the 3rd node automatically backup … or maybe you just
> need 2 copies with self-replication … if one goes down

Right shape, wrong number for production. The math:

| Strategy | Storage overhead | Real-world durability | Vulnerable window |
|---|---|---|---|
| 2 copies + auto-repair | 100% | ~6 nines | 20–30 min repair window — single failure during repair = data loss |
| 3 copies + auto-repair | 200% | ~10 nines | Triple failure during repair window |
| **Reed-Solomon 10/14** | **40%** | **~11 nines** | Need 5+ shard losses simultaneously |
| Reed-Solomon 6/9 | 50% | ~10 nines | Need 4+ shard losses |

2 copies is too thin in any network with non-trivial churn — the
repair window leaves you single-replica for ~20–30 min per failure
event. Storj and Backblaze both publish RS coding for this reason.

For c0mpute: we offer **two tiers** — `cheap` (3-copy replication,
fits naive workers + simple ops) and `verified` (RS 10/14, requires
≥14 worker placement + storage challenges). Same plugin, different
job manifest opt-ins.

## Detailed design

### Architecture

```
                    ┌──────────────────────┐
   c0mpute job  ──▶ │  c0mpute (Rust)      │
                    │   axum gateway       │── HTTP ──▶ peer storage workers
                    │   storage role       │
                    │   c0mpute-store +    │
                    │   erasure-coding mod │
                    └──────────────────────┘
                              │
                              └─ filesystem-backed shard store
```

Same daemon. The storage role is one of the existing worker roles
(`storage`, `transcode`, `gateway`, `verifier`). HTTP endpoints
extend the existing axum gateway.

### Reed-Solomon parameters

| Param | Value | Why |
|---|---|---|
| `k` (data shards) | 10 | Standard for 10/14; matches Storj's default |
| `n` (total shards) | 14 | 4 parity shards |
| Shard size | 256 KiB target | Big enough to amortize HTTP overhead, small enough to recover quickly |
| Block size | up to 2.5 MiB per coding pass | k × shard_size; large objects split into multiple stripes |
| Hash | blake3 per-shard + per-object | Object hash = blake3(plaintext); shard hash = blake3(shard_bytes) |

The `reed-solomon-erasure` Rust crate handles the math. We wrap it.

### HTTP surface (extension to existing gateway)

```
PUT  /storage/v1/objects/<object-hash>
       body: full plaintext bytes
       resp: { object_hash, shards: [{ index, hash, host_hint }] }

GET  /storage/v1/objects/<object-hash>
       resp: full plaintext bytes (reconstructed from shards)

GET  /storage/v1/shards/<shard-hash>
       resp: raw shard bytes (this node only)

PUT  /storage/v1/shards/<shard-hash>
       body: raw shard bytes
       header: x-c0mpute-object: <object-hash>
       header: x-c0mpute-shard-index: 0..13
       (used by peer placement; signed-request envelope auth)

POST /storage/v1/repair/<object-hash>
       admin/auto endpoint: regenerate missing shards from surviving
       set, announce replacements via gossip
```

Auth on PUTs / repair: signed-request envelopes per DIP-0007
(CoinPay DID).

### Shard placement (cross-node — Phase 2)

For v1 single-node MVP we encode locally and store all shards on the
same node — useful for bit-rot detection, pre-distribution staging,
and testing. **Not durable until we have peer placement.**

Phase 2 adds peer placement once `c0mpute-net` (libp2p) is wired:

- Choose 14 distinct workers, weighted by reputation × capability
  match × ASN/region diversity.
- PUT each shard via HTTP to its assigned worker.
- Track placement in a manifest stored at the requesting node and
  optionally announced via gossipsub.
- On retrieval: parallel-fetch ≥10 of 14 shards, race to decode.

### Auto-repair

Trigger: a node trying to GET/decode an object discovers >4 shards
unreachable, OR a periodic scan finds an object below threshold.

Repair flow:

1. Fetch the 10 surviving shards.
2. Reconstruct the original bytes.
3. Re-encode to 14 fresh shards (same RS params).
4. PUT replacements for the missing shards to fresh peers.
5. Update the object's placement manifest.
6. Sign a "repair completed" attestation through CoinPay (so the new
   shard hosts get credit and the failing host's reputation
   decrements — same primitive as job receipts).

### Tiers exposed in the job manifest

```json
{
  "storage": {
    "tier": "verified",        // "cheap" | "verified" | "private"
    "object_hash": "blake3:..." // returned after PUT; referenced after
  }
}
```

| Tier | Scheme | Encryption |
|---|---|---|
| `cheap` | 3-copy replication | optional |
| `verified` | RS 10/14 | server-side at-rest |
| `private` | RS 10/14 + customer-encrypted before PUT | customer-managed key (E2E; workers see ciphertext only) |

`private` reuses the original Quest PRD's E2E approach: customer
encrypts before PUT, network never sees plaintext, workers only do
RS encoding on the already-encrypted bytes.

### Pricing target

Match Storj's $0.004/GB-month for storage + $0.007/GB egress. The
40% RS overhead means workers actually allocate 1.4 GB per 1 GB
billed. At target rates that still leaves room for verification +
churn overhead, but only just — pricing must be revisited once we
have real numbers.

## Alternatives considered

**Skip storage entirely (DIP-0012 v1, withdrawn).** That was the
original recommendation. Reverted because (a) the user's
self-replicating-on-failure intuition is broadly right and worth
implementing, (b) RS 10/14 actually solves the cents-on-the-dollar
problem if we get the details right, (c) c0mpute jobs naturally
produce output that customers want stored — punting that to BYOS3
forever leaves a real product gap.

**3-copy replication only.** Simpler to implement but 200% overhead
torches the price target. Keep as the `cheap` tier, not the default.

**Wrap an existing network (Storj / Filecoin).** Faster to ship, but
their economic models are tied to their own tokens / networks; we'd
be reselling. Plugin model still allows a wrapper plugin for users
who want it, but we build our own primary scheme.

**Pure libp2p data plane (no HTTP).** Cleaner for fully-decentralized
purity. Worse for interop — browsers, curl, integrations all speak
HTTP natively. Mirrors infernet-protocol's choice for the same
reason.

## Migration & rollout

This DIP supersedes the previous "no storage network" stance. Kept
in the same DIP number (0012) since it never shipped to anyone yet.

Implementation phases:

**Phase 1 (this commit + follow-ups)** — single-node MVP:
- `c0mpute-store::erasure` module wraps `reed-solomon-erasure`.
- `Storage` struct in c0mpute-store handles encode → write 14 shards
  to local fs → manifest → reconstruct on read.
- `c0mpute-gateway` adds the HTTP endpoints (objects PUT/GET, shards
  GET, /repair/ stub).
- Tests covering encode-decode round-trip + corruption tolerance.

**Phase 2** — cross-node replication:
- Depends on `c0mpute-net` libp2p implementation (DIP-0010).
- Shard placement across peers using gossipsub announcements.
- Auto-repair daemon scanning for under-replicated objects.

**Phase 3** — economics:
- Storage challenges (proof-of-replication-lite per original PRD §14).
- Storage earnings / billing through CoinPay.
- Public marketplace pricing.

## Open questions

- **Default tier.** Probably `verified` (RS 10/14). `cheap` is opt-in
  for users who explicitly want it.
- **Encryption-at-rest scheme.** Per-shard or per-object? AES-GCM
  is the obvious choice; key management is the harder question.
- **Manifest hosting.** A small JSON saying "object X = these 14
  shards on these 14 nodes" needs durable hosting itself. Probably
  the manifest itself becomes a shard set, recursively.
- **Garbage collection.** When does a worker delete a shard? On
  TTL expiry? On the customer's storage subscription lapsing? Needs
  CoinPay billing integration to answer.

## Out of scope

- Permanent / Arweave-style storage. Our model is paid + ongoing,
  not pay-once.
- Filesystem-style mutable objects. Content-addressed, immutable.
  Mutability is a layer above (CRDT / mutable ref store) that uses
  storage but isn't storage.
- IPFS interop. Not in v1; potentially a separate plugin.
