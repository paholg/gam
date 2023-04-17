#!/usr/bin/env bash

set -euo pipefail

case $1 in
  gam)
    cargo run --features bevy/dynamic_linking "${@:2}"
  ;;
  debug)
    RUST_BACKTRACE=1 cargo run --features bevy/dynamic_linking --features debug "${@:2}"
  ;;
  train)
    cargo run --no-default-features --features bevy/dynamic_linking --features train --release "${@:2}"
  ;;
  *)
    echo "Invalid argument"
    # exit 1
  ;;
esac
