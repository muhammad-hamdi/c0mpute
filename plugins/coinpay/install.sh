#!/usr/bin/env sh
# coinpay plugin installer.
#
# Convention: subprocess plugins install from <homepage>/install.sh, where
# <homepage> is the value of `homepage` in plugins/coinpay/module.toml.
# Routing through c0mpute.com (https://c0mpute.com/plugins/coinpay/install.sh)
# keeps the c0mpute install URL stable; the chain target tracks the
# manifest's homepage.
#
# Override via $COINPAY_INSTALL_URL for testing or local mirrors.
# Source: https://github.com/profullstack/c0mpute/tree/master/plugins/coinpay
set -eu

UPSTREAM="${COINPAY_INSTALL_URL:-https://coinpayportal.com/install.sh}"

printf '\033[1;36m→\033[0m installing coinpay via %s\n' "$UPSTREAM"
exec sh -c "$(curl -fsSL "$UPSTREAM")" "$@"
