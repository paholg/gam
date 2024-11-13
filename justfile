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
