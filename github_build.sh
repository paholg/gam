#!/usr/bin/env bash

set -euo pipefail

target="$1"

echo "Building for $target"
cargo build --release --locked --target "$target" --bin client

mkdir "gam"
cp "target/$target/release/client" "gam/"
cp -r assets/ "gam/"
7z a -tzip "gam-$target.zip" "gam/"
rm -r "gam"
