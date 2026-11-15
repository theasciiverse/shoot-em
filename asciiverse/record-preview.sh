#!/bin/sh
# Records asciiverse/shoot-em.cast, the catalog preview, at 100x30
# (hub/RECORDING.md): the computer plays (--demo) in groove mode (--groove)
# for about 50 seconds, with sound off. Uses hub/tools/castrec.py from a sibling hub checkout.
#
#   sh asciiverse/record-preview.sh [seed]
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
castrec=${CASTREC:-$root/../../hub/tools/castrec.py}
seed=${1:-33}
state=$(mktemp -d)
trap 'rm -rf "$state"' EXIT
(cd "$root" && cargo build --release)
XDG_STATE_HOME=$state python3 "$castrec" "$root/asciiverse/shoot-em.cast" "$root" "w52" \
  -- "$root/target/release/shoot-em" --demo --groove --mute --seed "$seed"
