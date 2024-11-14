client *args:
	@just cargo run --bin client -- {{args}}

check *args:
	@just cargo check {{args}}

profile:
	CARGO_PROFILE_RELEASE_DEBUG=true just cargo flamegraph --bin client

update:
	nix flake update
	cargo update

fix:
	@just cargo clippy --fix --allow-staged

cargo cmd *args:
	cargo --color always {{cmd}} --features bevy/dynamic_linking {{args}}
	# cargo --color always {{cmd}} {{args}}

package_client target:
	#!/usr/bin/env bash
	set -euo pipefail

	echo "Building for {{target}}:"
	BIN=$(cargo build --message-format=json --release --locked --target {{target}} --bin client | jq -r 'select(.reason == "compiler-artifact" and .executable != null) | .executable') 
	mkdir "gam"
	cp "$BIN" "gam/gam"
	cp -r assets/ "gam/"
	7z a -tzip "gam-{{target}}.zip" "gam/"
	rm -r "gam"
