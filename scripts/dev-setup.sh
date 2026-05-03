#!/usr/bin/env sh
# Contributor bootstrap. Idempotent — safe to re-run.
#
# Operators don't need this. See dips/0004-toolchain-mise.md.
set -eu

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

say()  { printf '\033[1;36m→\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m✗\033[0m %s\n' "$*" >&2; exit 1; }

ensure_mise() {
  if command -v mise >/dev/null 2>&1; then
    say "mise found: $(mise --version)"
    return
  fi

  say "installing mise (https://mise.run)"
  if ! command -v curl >/dev/null 2>&1; then
    die "curl is required to install mise"
  fi

  curl -fsSL https://mise.run | sh

  if [ -x "$HOME/.local/bin/mise" ]; then
    PATH="$HOME/.local/bin:$PATH"
    export PATH
  else
    die "mise install completed but binary not found at \$HOME/.local/bin/mise"
  fi

  warn "mise installed. Add this to your shell rc to make it permanent:"
  warn '  eval "$($HOME/.local/bin/mise activate ${SHELL##*/})"'
}

ensure_mise

say "installing pinned toolchain from .mise.toml"
mise install

say "ensuring mise is trusted for this repo"
mise trust >/dev/null 2>&1 || true

say "installing JS workspace deps"
mise exec -- bun install

cat <<EOF

Done.

Useful tasks:
  mise run cli -- video doctor       # smoke-test the depin binary
  mise run dev-api                    # coordinator API on :8787
  mise run dev-web                    # dashboard on :3000/video
  mise run test                       # rust + ts checks

If you don't want to type 'mise exec' every time, activate it in your shell:
  echo 'eval "\$(\$HOME/.local/bin/mise activate \${SHELL##*/})"' >> ~/.\$(basename "\$SHELL")rc

EOF
