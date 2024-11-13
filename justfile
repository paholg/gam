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

rustfmt_config := replace("""
	imports_granularity=Crate,
	group_imports=StdExternalCrate,
	wrap_comments=true,
	format_code_in_doc_comments=true,
	format_strings=true
""", "\n", "")
fmt:
	@just cargo fmt -- --config {{rustfmt_config}}

[private]
cargo cmd *args:
	cargo --color always {{cmd}} --features bevy/dynamic_linking {{args}}
	# cargo --color always {{cmd}} {{args}}
