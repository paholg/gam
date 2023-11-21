#!/usr/bin/env bash

set -euo pipefail

# For Linux:
sudo apt-get update
sudo apt-get install --no-install-recommends libasound2-dev libudev-dev

# For Windows:
sudo apt-get install mingw-w64

targets=("x86_64-unknown-linux-gnu" "x86_86-pc-windows-gnu")

for target in "${targets[@]}"; do
  echo "Building for $target"
  cargo build --release --locked --target "$target" --bin client

  mkdir "gam"
  cp "target/$target/release/client" "gam/"
  cp -r assets/ "gam/"
  7z a -tzip "gam-$target.zip" "gam/"
  rm -r "gam"
done
