---
dip: 0012
title: "c0mpute is compute-only; storage is BYOS3"
status: Accepted
authors:
  - anthony@profullstack.com
created: 2026-05-03
updated: 2026-05-03
discussion:
implementation: customer storage URLs in job manifests; BYOS plugin (DIP-0013)
supersedes:
superseded-by:
---

## Summary

c0mpute is a **compute marketplace**, not a storage network. Customers
bring their own storage (R2, B2, S3, IPFS, anything addressable by
URL) for both inputs and outputs. Workers hold data only ephemerally
during a job.

We are **not** building a price-competitive p2p storage network and
this DIP rejects the design that would have done so. See DIP-0013 for
the BYOS3 integrations that let customers plug in storage cheaply.

## Motivation

This DIP went through two drafts. The first said "no storage" —
correct conclusion. The second tried to argue we could ship Reed-
Solomon 10/14 + auto-repair and hit cents-per-GB. That second draft
was wrong, and reviewer pushback called it out:

> it doesn't scale. i worked at a company that was wanting to do p2p
> file hosting and they said ipfs and all the other native crypto
> ones were garbage because they couldn't compete with s3 pricing

The reviewer was right and matches everyone else's conclusion.
Reverting to the original position with the math committed to record:

### Why p2p storage can't beat S3-class pricing

1. **S3-class providers sell at near-cost on integrated infrastructure
   spend.** Cloudflare R2 ($0.015/GB + $0 egress), Backblaze B2
   ($0.006/GB), AWS Glacier Deep Archive ($0.00099/GB-month) — these
   prices come from dedicated peering, optimized hardware lifecycle,
   power/cooling at hyperscale. Not from clever protocols.

2. **A p2p network of consumer/prosumer nodes can't reach those
   per-GB economics**, no matter the coding scheme. Storj at $0.004
   only works because operators effectively donate spare capacity AND
   Storj runs centralized metadata under the hood. Filecoin "wins"
   on storage cost but retrieval is slow and unreliable for active
   workloads.

3. **Hidden overheads consume any nominal margin:**
   - Verification (proof-of-replication compute)
   - Repair bandwidth on node churn
   - Coordinator operations (placement, manifest tracking)
   - Sybil prevention / trust establishment
   - Customer-side latency tax (worse UX vs. CDN-served S3)

4. **Where p2p storage genuinely wins, it isn't on price.**
   Sovereignty (data jurisdiction), customer-held encryption keys,
   censorship resistance, compute-locality. Real but small markets;
   not a flagship product.

The honest conclusion: the only sensible storage plan for c0mpute is
**don't run a storage network**.

## Detailed design

### v1 storage model: customer-provided URLs

Job manifests reference inputs and outputs by URL:

```json
{
  "input": {
    "uri": "https://customer-bucket.r2.cloudflarestorage.com/input.mov?signed=...",
    "sha256": "..."
  },
  "output": {
    "uri": "https://customer-bucket.r2.cloudflarestorage.com/output.mp4?signed=...",
    "format": "mp4"
  }
}
```

The worker:

1. Downloads input from the signed URL.
2. Verifies the sha256 (customer commits a hash so the worker can
   detect tampering).
3. Runs the workload locally on ephemeral disk.
4. Uploads output to the customer-provided destination URL.
5. Signs a receipt covering input hash + output hash + runtime image
   hash, returns through CoinPay.

Working data on the worker is ephemeral.

### What we keep from the brief storage detour

- **`c0mpute-store` Rust crate** stays. It's per-node ephemeral cache
  for working data — not the basis of a storage network.
- **`c0mpute-store::erasure`** (Reed-Solomon 10/14 in ~150 LOC) stays
  as **niche capability**, not a product. Use cases: bit-rot
  protection on long-running worker chunks; sovereignty-tier customers
  who want encrypted-at-rest output stored locally on workers (an
  edge-case feature, not the default path); future cold-archive
  experiments.
- **`c0mpute-store::storage::Storage`** wrapper stays for the same
  niche cases. Enabling it is opt-in via job-manifest tier and is
  not exposed as a marketed storage product.

### What this DIP rules out

- A `storage` worker role marketed at general-purpose customers.
- Pricing claims like "cheaper than S3".
- Building a CDN out of c0mpute gateway nodes.
- Cross-node Reed-Solomon shard distribution as a v1 deliverable.
- Any storage-as-a-service positioning on c0mpute.com.

### What replaces it

DIP-0013: **BYOS plugin** — first-class integrations with R2 / B2 /
S3 / Backblaze / IPFS gateways / Storj S3 gateway. Customer connects
their storage credentials once in the dashboard or via
`c0mpute coinpay storage link <provider>`; jobs reference outputs by
the customer's chosen prefix; c0mpute uploads land there
automatically. We don't host bytes; we make hosting them elsewhere
frictionless.

### When storage might come back as a product

A future c0mpute revisit could justify building a storage network
only if **all three** of these are true:

1. **A specific market with non-price-driven needs is paying for
   c0mpute compute** (sovereignty, encryption-at-rest with E2E,
   censorship resistance).
2. **Existing networks (Storj / Filecoin / Arweave) don't fit** that
   market's exact requirements.
3. **The economics on that segment can sustain the operational
   overhead** without trying to beat S3 on $/GB.

Until all three are true, keep storage out.

## Alternatives considered

**Build it anyway with RS 10/14 + auto-repair.** This was the
in-flight v2 of this DIP, withdrawn after reviewer pushback. The
math doesn't survive S3-class price competition — see Motivation.

**Wrap an existing decentralized network (Storj / Filecoin).** Better
than building our own, but they have their own economics and tokens.
We'd be reselling. The BYOS plugin in DIP-0013 already includes
optional integrations with those networks for customers who want them.

**Sovereignty-only storage as a premium product.** Genuine demand
exists (legal, healthcare, defense). But it doesn't justify a
network-wide storage layer. If a single customer pays for it, build
a bespoke deployment for them, don't build a marketplace product.

## Migration & rollout

Performed in this commit:

- DIP-0012 v2 (the "build it" draft) is withdrawn before any
  marketing or external promise of a storage product.
- The Phase 1 code (`c0mpute-store::erasure`,
  `c0mpute-store::storage::Storage`, manifest types) stays in the
  tree for niche / future-experiment use, with module docs updated
  to clarify it's not a marketed product.
- HTTP shard endpoints in `c0mpute-gateway` (planned but not yet
  written) — cancelled.
- DIP-0013 (BYOS3 plugin) is the actual storage-shaped work.

## Open questions

- **Receipt anchoring.** Job receipts (signed by worker + validator
  DIDs) need somewhere durable. CoinPay's own ledger is the assumed
  home — verifying that assumption is a CoinPay-side question.
- **Public attestation log.** If customers want a public "yes, c0mpute
  job XYZ ran successfully" surface, we'd want some persistent
  destination. Probably CoinPay receipts → a public aggregator. Out
  of scope here.

## Out of scope

- Reviving the storage-network idea in any form on c0mpute.com.
- Permanent / Arweave-style storage.
- Filesystem-style mutable objects.
