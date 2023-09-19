#!/usr/bin/env bash

set -euo pipefail

version=$1
target=$2

dir="gam-$version"
target="gam-$version-$target.zip"

echo "TARGET=$target" >> "$GITHUB_ENV"

mkdir "$dir"
cp "target/$target/release/gam" "$dir/"
cp -r assets/ "$dir/"
7z a -tzip "$target" "$dir/"
