---
dip: 0001
title: "All public surfaces under depin.quest/video"
status: Superseded
authors:
  - anthony@profullstack.com
created: 2026-05-03
updated: 2026-05-03
discussion:
implementation: scaffolded in initial repo (apps/web basePath, apps/coordinator BASE_PATH, scripts/install.sh)
supersedes:
superseded-by: 0005
---

> **Superseded by DIP-0005** (c0mpute.com rebrand). The `/video` namespace
> under depin.quest is no longer the project's public surface. Video is now
> a workload module under `c0mpute.com`, with FFmpeg transcoding routes
> served from `c0mpute.com/api/...` rather than `depin.quest/video/...`.
> The motivation behind this DIP — protect the parent brand from being
> tied to one product line — still applies and informs DIP-0005.

## Summary

Mount every public surface for the video product line under the `/video`
path on `depin.quest`: dashboard at `/video`, REST API at `/video/api/v1`,
install script at `/video/install.sh`, release artifacts at
`/video/releases/...`. Keep the apex (`depin.quest/`) reserved for the
parent brand and future product lines (`/storage`, `/compute`, etc.).

## Motivation

We expect to ship more product lines under the depin.quest brand. If video
is mounted at the apex now, we paint ourselves into either (a) a subdomain
sprawl (`storage.depin.quest`, `compute.depin.quest`) — which complicates
TLS, cookies, and SSO — or (b) a renaming exercise that breaks every install
script and embed already in the wild.

Choosing the path namespace upfront costs us nothing today and avoids both
problems.

## Detailed design

| Surface              | Path                                              |
|----------------------|---------------------------------------------------|
| Marketing site       | `/video`                                          |
| Dashboard (auth'd)   | `/video/app/...`                                  |
| Embed iframe         | `/video/embed/<videoId>`                          |
| Coordinator REST     | `/video/api/v1/...`                               |
| CoinPayments IPN     | `/video/api/v1/webhooks/coinpayments`             |
| Install script       | `/video/install.sh`                               |
| Release artifacts    | `/video/releases/<version>/depin-<os>-<arch>.tar.gz` |
| Known-issues feed    | `/video/api/v1/known-issues`                      |
| Release manifest     | `/video/api/v1/releases/latest`                   |

Implementation specifics:

- Next.js dashboard: `next.config.ts` sets `basePath: "/video"` and
  `assetPrefix: "/video"`.
- Coordinator: Hono app mounts the v1 router at `${BASE_PATH}/api/v1`,
  defaulting `BASE_PATH=/video`. Health endpoint also mirrored at apex
  `/health` for load-balancer probes.
- Install script: hardcodes `https://depin.quest/video/releases` as the
  default; overridable via `DEPIN_RELEASE_BASE` env var for testing.
- Self-upgrade: nodes poll `https://depin.quest/video/api/v1/releases/latest`.

## Alternatives considered

**Subdomains (`video.depin.quest`).** Cleaner separation, but: extra TLS
cert per line, cross-origin cookie hassles, more DNS to maintain, and worse
SSO ergonomics if we add an account model that spans lines.

**Apex-only with eventual rename when line #2 ships.** Cheapest now,
catastrophic when we have customers depending on the apex paths. We pay the
migration cost forever.

**Per-line repos.** Doesn't help — we'd still need URL discipline. Repo
layout is a separate question (see DIP-future).

## Migration & rollout

Greenfield — no migration needed for v1. Future product lines pick a sibling
path (`/storage`, `/compute`) and follow the same conventions established
here.

## Open questions

None at acceptance. The marketing apex (`depin.quest/`) has not been
designed yet; that's a separate decision and doesn't block this DIP.

## Out of scope

- The marketing site at `depin.quest/` (apex).
- DNS, CDN, or edge config decisions — those follow once paths are stable.
- API versioning beyond `/v1` — handled in a future DIP if we ever need it.
