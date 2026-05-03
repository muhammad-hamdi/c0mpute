#!/usr/bin/env sh
# infernet plugin installer.
#
# Convention: subprocess plugins install from <homepage>/install.sh, where
# <homepage> is the value of `homepage` in plugins/infernet/module.toml.
# Routing through c0mpute.com keeps the c0mpute install URL stable; the
# chain target tracks the manifest's homepage.
#
# Override via $INFERNET_INSTALL_URL for testing or local mirrors.
# Source: https://github.com/profullstack/c0mpute/tree/master/plugins/infernet
# Upstream: https://github.com/infernetprotocol/infernet-protocol
set -eu

UPSTREAM="${INFERNET_INSTALL_URL:-https://infernetprotocol.com/install.sh}"

printf '\033[1;36m→\033[0m installing infernet via %s\n' "$UPSTREAM"
exec sh -c "$(curl -fsSL "$UPSTREAM")" "$@"
