#!/usr/bin/env bash

set -euo pipefail

version=$1
target=$2

dir="gam-$version"
out_file="gam-$version-$target.zip"

echo "TARGET=$out_file" >> "$GITHUB_ENV"

mkdir "$dir"
cp "target/$target/release/client" "$dir/"
cp -r assets/ "$dir/"
7z a -tzip "$out_file" "$dir/"
