#!/usr/bin/env bash

set -euo pipefail

case $1 in
  client)
    BEVY_ASSET_ROOT="./" cargo run --features bevy/dynamic_linking --bin client
  ;;
  debug)
    BEVY_ASSET_ROOT="./" RUST_BACKTRACE=1 cargo run --features bevy/dynamic_linking --bin client --features debug
  ;;
  *)
    echo "Invalid argument"
    # exit 1
  ;;
esac
