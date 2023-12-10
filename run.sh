#!/usr/bin/env bash

set -euo pipefail

case $1 in
  client)
    BEVY_ASSET_ROOT="./" cargo run --bin client --features bevy/dynamic_linking -- "${@:2}"
  ;;
  profile)
    BEVY_ASSET_ROOT="./" CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin client
  ;;
  *)
    echo "Invalid argument"
    exit 1
  ;;
esac
