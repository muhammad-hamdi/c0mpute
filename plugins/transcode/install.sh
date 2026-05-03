#!/usr/bin/env sh
# transcode plugin installer.
#
# transcode is an in-process plugin compiled into the `c0mpute` binary
# itself — there's nothing to install separately. Running this script
# tells you that and points at the c0mpute installer.
#
# Served at https://c0mpute.com/plugins/transcode/install.sh.
set -eu

cat <<EOF
transcode is built into the c0mpute binary — no separate install needed.

Install c0mpute (which includes transcode) with:
  curl -fsSL https://c0mpute.com/install.sh | sh

Submit a transcode job:
  c0mpute transcode submit input.mov --preset hls
EOF
