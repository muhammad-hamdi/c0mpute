#!/usr/bin/env sh
# coinpay plugin installer.
#
# Chains to the upstream coinpay installer. Routing through c0mpute.com
# keeps the c0mpute install URL stable even if upstream rotates theirs.
#
# Served at https://c0mpute.com/plugins/coinpay/install.sh.
# Source: https://github.com/profullstack/c0mpute/tree/master/plugins/coinpay
set -eu

UPSTREAM="${COINPAY_INSTALL_URL:-https://coinpayportal.com/install.sh}"

printf '\033[1;36m→\033[0m installing coinpay via %s\n' "$UPSTREAM"
exec sh -c "$(curl -fsSL "$UPSTREAM")" "$@"
