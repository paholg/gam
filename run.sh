#!/usr/bin/env bash

set -euo pipefail

case $1 in
  gam)
    cargo run --features bevy/dynamic --bin gam "${@:2}"
  ;;
  headless)
    cargo run --no-default-features --features bevy/dynamic --bin headless "${@:2}"
  ;;
esac
