#!/usr/bin/env bash

set -euo pipefail

case $1 in
  gam)
    cargo run --features bevy/dynamic --bin gam "${@:2}"
  ;;
  train)
    cargo run --no-default-features --features bevy/dynamic --features train --release --bin train "${@:2}"
  ;;
  *)
    echo "Invalid argument"
    # exit 1
  ;;
esac
