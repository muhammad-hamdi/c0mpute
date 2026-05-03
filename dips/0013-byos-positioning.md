---
dip: 0013
title: "Position: GPU batch-compute marketplace; storage is BYOS"
status: Accepted
authors:
  - anthony@profullstack.com
created: 2026-05-03
updated: 2026-05-03
discussion:
implementation:
supersedes:
superseded-by:
---

## Summary

c0mpute's product position: **GPU batch-compute marketplace**.

We compete with the cloud — but only on workloads where consumer
hardware has a real arbitrage (transcode, AI inference, ML data
prep, diffusion image gen). 5–8× cheaper than AWS / Mux / Replicate
on those workloads.

We **don't** compete on storage, always-on services, low-latency
request/response, or general-purpose VMs. Customers bring their own
storage (BYOS) — c0mpute provides first-class integrations with R2,
B2, S3, IPFS, Storj-S3-gateway, etc.

## Motivation

The honest framing emerged after rejecting the "build a storage
network that beats S3 on price" idea (see DIP-0012 v2 → v3). The
underlying question:

> if we can compete with cloud hosting that would be sick

We can — *on a specific subset of cloud workloads*. The pricing math
isn't magic; it's the consumer-GPU arbitrage:

- Hyperscalers buy datacenter GPUs (H100 ~$40k each), rack them with
  redundant power/cooling, amortize across enterprise SLAs.
- Consumer GPUs (RTX 4090, RTX 5090, Apple M-series) deliver ~70% the
  perf at ~5% the cost.
- Their owners run them maybe 2–4 hours/day. The rest is idle.
- c0mpute monetizes the idle cycles.

This is a concrete arbitrage, not a "decentralization is better"
argument.

## Detailed design

### Where we win on price

| Workload | Cloud price | c0mpute target | Multiple |
|---|---|---|---|
| 1080p H.264 transcode | Mux $0.04/min | $0.005/min | 8× |
| 1080p AV1 transcode | Mux $0.07/min | $0.012/min | 6× |
| LLM inference (8B) | Together AI ~$0.20/M tok | $0.04/M tok | 5× |
| ML data prep (frame extract / chunk / hash) | AWS MediaConvert $0.025/min | $0.003/min | 8× |
| Diffusion image gen | Replicate $0.005/img | $0.001/img | 5× |

Common shape across these: **batch, GPU-bound, no always-on
availability requirement, fits a job-manifest model.**

### Where we don't compete

| Workload type | Why we lose | What customers should use |
|---|---|---|
| Object storage at $/GB | Hyperscaler infra-cost basis (DIP-0012) | R2, B2, S3, Storj |
| Always-on services (DB, web app) | c0mpute is job-based, not service-based | Fly.io, Railway, RDS, Vercel |
| Low-latency req/resp | p2p latency tax | Cloudflare Workers, Lambda@Edge |
| GP VMs | Datacenter VMs cheaper for steady-state | EC2, DigitalOcean Droplets, Hetzner |
| CDN / asset delivery | Hyperscaler peering at scale | Cloudflare, Fastly, Bunny |

We don't market to those use cases at all.

### BYOS — bring your own storage

Storage is **always** the customer's problem in v1+. The c0mpute
experience needs to make that frictionless:

1. **Job manifests reference storage by URL.** Inputs come from
   customer-signed URLs (any S3-compatible, IPFS gateway, plain
   HTTPS). Outputs go to a customer-provided destination URL.
2. **CLI helpers for the common cases:**
   ```sh
   c0mpute coinpay storage link r2 \
     --account-id <id> --access-key <ak> --secret-key <sk>
   c0mpute coinpay storage list
   c0mpute coinpay storage default r2:my-bucket/outputs/
   ```
   Linked credentials are encrypted, kept in `~/.config/coinpay/`,
   and used to mint signed PUT URLs at job-submit time. Workers
   never see the credentials, only the signed URL.
3. **Integrations targeted for v1:**
   - Cloudflare R2 (cheapest with $0 egress)
   - Backblaze B2
   - AWS S3
   - Generic S3-compatible (MinIO, Wasabi, etc.)
   - Storj S3 gateway (decentralized for customers who want it)
   - IPFS public gateways (read-only inputs)
   - Local file paths (for `c0mpute` running on the customer's own
     machine — useful for the small/dev case)

### Pricing implementation

`c0mpute coinpay pricing` shows current rates:

```
$ c0mpute coinpay pricing
Workload                           Cloud equivalent   c0mpute price
ffmpeg.transcode  h264 1080p       $0.04 / min        $0.005 / min  (8×)
ffmpeg.transcode  av1 1080p        $0.07 / min        $0.012 / min  (6×)
infernet.inference  8B params      $0.20 / M tok      $0.04  / M tok (5×)
...
```

Backed by a simple price feed at `c0mpute.com/pricing.json` (static,
versioned, signed). Worker bids must beat the public ceiling minus
the network's 20% take rate.

### Scheduler implications

The scheduler optimizes for **price × hardware fit × reputation**.
Specifically:

- A customer's job manifest declares max price per output unit.
- Workers advertise capability + spot price per unit.
- Eligible workers (capability match + price ≤ max + reputation tier
  ≥ required) bid. Lowest bidder wins, weighted by reputation.
- For batch jobs, the same matchmaking happens per-chunk, so a job's
  effective cost is the sum of accepted bids.

This is the existing job-dispatch flow from DIP-0011 (gossipsub-based
auction); no new design needed.

### What "compete with cloud" means concretely

Customers should be able to:

1. Run a job at 5–8× lower cost than the cloud equivalent.
2. With latency that matches batch / async workloads (seconds to
   minutes per job, not 10s of milliseconds per request).
3. With BYO storage so they aren't locked into ours.
4. With a clean CLI / SDK that feels like Replicate or Modal, not
   like a blockchain.

We do *not* claim to replace AWS. We claim to be cheaper than AWS
for a specific bucket of workloads, with no commitment penalty.

## Alternatives considered

**Position as "AWS replacement".** Marketing line that doesn't
survive contact with customer questions ("can I run a Postgres on
this?" — no). Hurts trust.

**Position purely as transcode network.** Returns us to the original
Quest pitch — too narrow now that infernet exists as a peer module
and there's clear demand for AI inference batches.

**Don't claim cloud comparison at all.** Possible but customers
want a concrete cost framing to reason about us. "5–8× cheaper than
Mux for H.264" is a recognizable shape. No cost framing means no
sale.

## Migration & rollout

This DIP locks the marketing/positioning. Implementation:

- **`c0mpute.com/pricing` page** — table of supported workloads with
  current c0mpute target prices and the cloud-equivalent prices we
  beat. Source-of-truth: a static `pricing.json` in the repo,
  rendered on the marketing site.
- **`c0mpute coinpay storage link <provider>`** CLI verbs (and the
  underlying credential storage) — initial implementation in
  coinpay's repo, with c0mpute providing the surface area.
- **BYOS3 documentation** — `c0mpute.com/docs#byos` section
  explaining the integrations and showing the common job-manifest
  shapes.

## Open questions

- **Take rate.** PRD §15 set 20%. Re-evaluate once we have real
  bidding data.
- **Worker payouts.** CoinPay-side concern.
- **Egress cost coverage.** When a customer uploads to their own R2,
  egress is their problem. When a worker uploads to the customer's
  R2, that egress is the *worker's* problem. Need to fold that into
  the price formula.

## Out of scope

- Building any compute or storage product outside the GPU-batch
  niche. Specifically: no managed databases, no always-on services,
  no edge functions, no CDN.
- Detailed CoinPay credential schema — that's in coinpay's repo.
