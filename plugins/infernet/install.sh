#!/usr/bin/env sh
# infernet plugin installer.
#
# Chains to the upstream infernet-protocol installer.
#
# Served at https://c0mpute.com/plugins/infernet/install.sh.
# Source: https://github.com/profullstack/c0mpute/tree/master/plugins/infernet
# Upstream: https://github.com/infernetprotocol/infernet-protocol
set -eu

UPSTREAM="${INFERNET_INSTALL_URL:-https://infernetprotocol.com/install.sh}"

printf '\033[1;36m→\033[0m installing infernet via %s\n' "$UPSTREAM"
exec sh -c "$(curl -fsSL "$UPSTREAM")" "$@"
