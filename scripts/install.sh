#!/usr/bin/env sh
# c0mpute.com installer. Served at https://c0mpute.com/install.sh.
# Installs the v1 CLI stack: c0mpute, coinpay, infernet.
# Idempotent — re-running upgrades in place.
#
# Usage:
#   curl -fsSL https://c0mpute.com/install.sh | sh
#
# Flags (per c0mpute v1 PRD §"Installer Modes"):
#   --minimal       Install only c0mpute
#   --no-coinpay    Skip CoinPay CLI
#   --no-infernet   Skip Infernet CLI
#   --worker        Add worker-readiness checks (Docker, FFmpeg)
#   --developer     Verbose diagnostics + dev tools
#   --force         Reinstall even if already present
set -eu

C0MPUTE_VERSION="${C0MPUTE_VERSION:-latest}"
C0MPUTE_HOME="${C0MPUTE_HOME:-$HOME/.c0mpute}"
RELEASE_BASE="${C0MPUTE_RELEASE_BASE:-https://c0mpute.com/releases}"

INSTALL_C0MPUTE=1
INSTALL_COINPAY=1
INSTALL_INFERNET=1
WORKER_MODE=0
DEVELOPER_MODE=0
FORCE=0

# ────────────────────────────────────────────────────────────────────────
# tiny stdout helpers
# ────────────────────────────────────────────────────────────────────────

say()  { printf '\033[1;36m→\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m✗\033[0m %s\n' "$*" >&2; exit 1; }
ok()   { printf '\033[1;32m✓\033[0m %s\n' "$*"; }

# ────────────────────────────────────────────────────────────────────────
# arg parsing
# ────────────────────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
  case "$1" in
    --minimal)      INSTALL_COINPAY=0; INSTALL_INFERNET=0 ;;
    --no-coinpay)   INSTALL_COINPAY=0 ;;
    --no-infernet)  INSTALL_INFERNET=0 ;;
    --worker)       WORKER_MODE=1 ;;
    --developer)    DEVELOPER_MODE=1 ;;
    --force)        FORCE=1 ;;
    --help|-h)
      sed -n '2,18p' "$0"
      exit 0
      ;;
    *)
      die "unknown flag: $1 (try --help)"
      ;;
  esac
  shift
done

# ────────────────────────────────────────────────────────────────────────
# platform detection
# ────────────────────────────────────────────────────────────────────────

detect_platform() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)
  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *) die "unsupported arch: $arch" ;;
  esac
  case "$os" in
    linux|darwin) ;;
    *) die "unsupported os: $os (Linux/macOS only; Windows users see docs)" ;;
  esac
  printf '%s-%s' "$os" "$arch"
}

require() {
  command -v "$1" >/dev/null 2>&1 || die "$1 is required but not installed"
}

# ────────────────────────────────────────────────────────────────────────
# install one CLI tarball
# ────────────────────────────────────────────────────────────────────────

install_one() {
  bin="$1"           # binary name (c0mpute / coinpay / infernet)
  platform="$2"

  target="$C0MPUTE_HOME/bin/$bin"
  if [ -x "$target" ] && [ "$FORCE" -eq 0 ]; then
    say "$bin already installed at $target (use --force to reinstall)"
    return 0
  fi

  artifact="${bin}-${platform}.tar.gz"
  url="${RELEASE_BASE}/${C0MPUTE_VERSION}/${artifact}"
  sig_url="${url}.minisig"
  tmp=$(mktemp -d)

  say "downloading $bin ${C0MPUTE_VERSION}"
  if ! curl -fsSL "$url" -o "$tmp/$artifact"; then
    rm -rf "$tmp"
    die "could not download $url"
  fi
  curl -fsSL "$sig_url" -o "$tmp/$artifact.minisig" 2>/dev/null \
    || warn "no signature published for $bin yet; continuing"

  if command -v minisign >/dev/null 2>&1 && [ -f "$tmp/$artifact.minisig" ]; then
    say "verifying signature for $bin"
    C0MPUTE_PUBKEY="${C0MPUTE_PUBKEY:-RWQ_REPLACE_ME_WITH_PROD_MINISIGN_PUBKEY}"
    if ! minisign -V -P "$C0MPUTE_PUBKEY" -m "$tmp/$artifact" -x "$tmp/$artifact.minisig" >/dev/null 2>&1; then
      rm -rf "$tmp"
      die "signature verification failed for $bin"
    fi
  fi

  tar -xzf "$tmp/$artifact" -C "$C0MPUTE_HOME/bin"
  chmod +x "$C0MPUTE_HOME/bin/$bin"
  rm -rf "$tmp"
  ok "installed $bin → $C0MPUTE_HOME/bin/$bin"
}

# ────────────────────────────────────────────────────────────────────────
# shell rc updates
# ────────────────────────────────────────────────────────────────────────

ensure_path() {
  for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
    [ -f "$rc" ] || continue
    if ! grep -q '\.c0mpute/bin' "$rc"; then
      printf '\n# Added by c0mpute installer\nexport PATH="$HOME/.c0mpute/bin:$PATH"\n' >> "$rc"
    fi
  done
}

# ────────────────────────────────────────────────────────────────────────
# post-install verification + diagnostics
# ────────────────────────────────────────────────────────────────────────

print_versions() {
  echo
  if [ "$INSTALL_C0MPUTE" -eq 1 ] && [ -x "$C0MPUTE_HOME/bin/c0mpute" ]; then
    printf 'c0mpute installed:  %s\n'  "$("$C0MPUTE_HOME/bin/c0mpute" version 2>/dev/null | tail -1)"
  fi
  if [ "$INSTALL_COINPAY" -eq 1 ] && [ -x "$C0MPUTE_HOME/bin/coinpay" ]; then
    printf 'coinpay installed:  %s\n'  "$("$C0MPUTE_HOME/bin/coinpay" version 2>/dev/null | tail -1)"
  fi
  if [ "$INSTALL_INFERNET" -eq 1 ] && [ -x "$C0MPUTE_HOME/bin/infernet" ]; then
    printf 'infernet installed: %s\n'  "$("$C0MPUTE_HOME/bin/infernet" version 2>/dev/null | tail -1)"
  fi
}

run_doctor() {
  if [ -x "$C0MPUTE_HOME/bin/c0mpute" ]; then
    PATH="$C0MPUTE_HOME/bin:$PATH" "$C0MPUTE_HOME/bin/c0mpute" doctor || true
  fi
}

worker_checks() {
  echo
  say "worker-readiness checks"
  if command -v docker >/dev/null 2>&1; then ok "docker present"; else warn "docker not installed (recommended for sandboxed jobs)"; fi
  if command -v ffmpeg >/dev/null 2>&1; then ok "ffmpeg present"; else warn "ffmpeg not installed (required for transcode jobs)"; fi
}

# ────────────────────────────────────────────────────────────────────────
# main
# ────────────────────────────────────────────────────────────────────────

main() {
  require curl
  require tar
  require uname

  platform=$(detect_platform)
  mkdir -p "$C0MPUTE_HOME/bin"

  if [ "$INSTALL_C0MPUTE" -eq 1 ]; then install_one c0mpute  "$platform"; fi
  if [ "$INSTALL_COINPAY" -eq 1 ]; then install_one coinpay  "$platform"; fi
  if [ "$INSTALL_INFERNET" -eq 1 ]; then install_one infernet "$platform"; fi

  ensure_path

  if [ "$WORKER_MODE" -eq 1 ]; then worker_checks; fi

  print_versions
  run_doctor

  cat <<EOF

Next steps:
  coinpay did create
  c0mpute worker register
  c0mpute doctor
  c0mpute worker start

If your shell isn't picking up the new binaries:
  export PATH="\$HOME/.c0mpute/bin:\$PATH"

Docs: https://c0mpute.com/docs
EOF

  if [ "$DEVELOPER_MODE" -eq 1 ]; then
    echo
    say "developer mode: env"
    echo "  C0MPUTE_HOME=$C0MPUTE_HOME"
    echo "  C0MPUTE_VERSION=$C0MPUTE_VERSION"
    echo "  RELEASE_BASE=$RELEASE_BASE"
    echo "  PLATFORM=$platform"
  fi
}

main "$@"
