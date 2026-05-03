# c0mpute

> Decentralized compute network. v1 ships with three CLIs — `c0mpute`,
> `coinpay`, `infernet` — and a transcode workload module powered by
> FFmpeg. See [`docs/c0mpute-v1.md`](docs/c0mpute-v1.md) for the v1 PRD.

```
c0mpute     generic p2p compute network CLI
coinpay     DID, wallet, escrow, payments, receipts, reputation
infernet    AI inference workload CLI / protocol
```

Install everything with one command:

```bash
curl -fsSL https://c0mpute.com/install.sh | sh
```

**Repo:** [github.com/profullstack/c0mpute](https://github.com/profullstack/c0mpute)

**Distribution policy:** binaries via `curl | sh` + self-update; modules
via signed tarballs on `c0mpute.com/modules`. **No npm publishing**, no
package sprawl. JS workspace packages are `private: true` for
internal-only `bun install` consumption. See
[DIP-0006](dips/0006-module-model.md).

Design proposals: [`dips/`](dips/).

The original Quest video PRD is preserved in [`docs/PRD.md`](docs/PRD.md);
its FFmpeg-specific content informs the **transcode module**.

---

## URL & CLI namespace

Both URL routes and CLI subcommands are organized by module id:

| Surface | Pattern | Example |
|---|---|---|
| Web/PWA | `c0mpute.com/<module>/...` | `c0mpute.com/transcode/install` |
| CLI | `c0mpute <module> <subcommand>` | `c0mpute coinpay did create` |
| Direct CLI | `<module> <subcommand>` | `coinpay did create` |
| API (planned) | `c0mpute.com/api/v1/<module>/...` | `c0mpute.com/api/v1/transcode/jobs` |

v1 modules: `transcode`, `coinpay`, `infernet`.

## Surfaces per module (web / PWA / desktop / CLI)

Each module ships across these surfaces:

- **CLI** — `c0mpute <module>` umbrella + standalone module binary.
- **Web / PWA** — Next.js app with `basePath = "/<module>"`.
- **Desktop (Electron)** — wraps the same Next.js bundle. Future:
  `apps/desktop`. Will load any installed module's web app.

## Repo layout

```
.
├── docs/
│   ├── c0mpute-v1.md                 # v1 augmentation PRD (current source of truth)
│   └── PRD.md                        # original Quest PRD (transcode module internals)
├── dips/                             # depin/c0mpute improvement plans
│   ├── README.md
│   ├── 0000-template.md
│   ├── 0001…0003                     # superseded
│   └── 0004…0007                     # accepted: mise, rebrand, modules, CoinPay DID
├── node/                             # Rust workspace producing c0mpute, coinpay, infernet
│   └── crates/
│       ├── quest-cli/                # produces `c0mpute`
│       ├── coinpay-cli/              # produces `coinpay`
│       ├── infernet-cli/             # produces `infernet`
│       ├── quest-core/               # supervisor + config
│       ├── quest-net/                # libp2p layer (scaffold)
│       ├── quest-store/              # content-addressed chunk store
│       ├── quest-transcode/          # FFmpeg orchestration
│       ├── quest-gateway/            # axum HTTP gateway role
│       ├── quest-verify/             # challenges + reputation
│       ├── quest-update/              # self-upgrade
│       ├── quest-doctor/             # self-diagnostics
│       ├── quest-proto/              # shared types
│       └── quest-api/                # coordinator HTTP client
├── apps/
│   ├── web/                          # @c0mpute/transcode-web — Next.js, basePath /transcode
│   └── coordinator/                  # @c0mpute/coordinator — Bun + Hono REST API
├── packages/
│   └── shared/                       # @c0mpute/shared — TS types
├── supabase/
│   └── migrations/0001_init.sql
├── .mise.toml                        # contributor toolchain pins (DIP-0004)
└── scripts/
    ├── install.sh                    # served at c0mpute.com/install.sh
    └── dev-setup.sh                  # contributor bootstrap
```

The internal Rust crate names start with `quest-` because the transcode
module's domain code came from the original Quest scaffold. They're
implementation details — what users see is `c0mpute`, `coinpay`, `infernet`.

## Quickstart

### First-time contributor setup

```bash
scripts/dev-setup.sh                  # mise + pinned tools + bun install
mise run cli -- doctor                # full-stack diagnostics
mise run coinpay -- did status
mise run infernet -- doctor
mise run test                         # rust unit tests + TS typechecks
```

Toolchain pins live in [`.mise.toml`](.mise.toml). See
[DIP-0004](dips/0004-toolchain-mise.md) for why this is contributor-only —
operators install static binaries, not a toolchain.

### Build the binaries

```bash
cargo build --manifest-path node/Cargo.toml --bins
./node/target/debug/c0mpute --help
./node/target/debug/coinpay --help
./node/target/debug/infernet --help
```

### Run the coordinator API

```bash
cp apps/coordinator/.env.example apps/coordinator/.env
# fill in SUPABASE_URL + SUPABASE_SERVICE_ROLE_KEY
bun run dev:coordinator
```

### Run the transcode dashboard

```bash
cp apps/web/.env.local.example apps/web/.env.local
bun run dev:web
# → http://localhost:3000/transcode
```

### Apply the Supabase schema

```bash
supabase db push
# or apply supabase/migrations/0001_init.sql via the SQL editor
```

## Status

Working today:

- Three CLIs build and run: `c0mpute`, `coinpay`, `infernet`
- `c0mpute <plugin> <subcommand>` plugin-style routing — transcode runs
  in-process, coinpay/infernet via subprocess passthrough
- `c0mpute doctor` cross-checks PATH for peer binaries
- `c0mpute modules list` shows the v1 module roster
- Coordinator boots, serves health + route stubs
- Transcode dashboard at `/transcode` with install instructions
- Supabase schema covers all original PRD §13 tables + RLS + atomic
  `claim_next_job()` RPC
- Installer (`scripts/install.sh`) installs all three CLIs with flag
  support per [DIP-0005](dips/0005-c0mpute-rebrand.md)

Not yet wired up:

- CoinPay DID generation, signing, escrow ([DIP-0007](dips/0007-coinpay-did-identity.md))
- Job manifest dispatch / scheduler
- libp2p transport
- Module marketplace UI on the dashboard
- `apps/coinpay-web`, `apps/infernet-web`, `apps/console` (cross-module),
  `apps/desktop` (Electron)

## License

Apache-2.0 across all our code.
