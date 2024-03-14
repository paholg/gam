client *args:
	cargo run --bin client --features bevy/dynamic_linking -- {{args}}

check:
	nix flake check

profile:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin client

update:
	nix flake update
	cargo update

fix:
	cargo clippy --fix --allow-staged

rustfmt_config := replace("""
	imports_granularity=Crate,
	group_imports=StdExternalCrate,
	wrap_comments=true,
	format_code_in_doc_comments=true,
	format_strings=true
""", "\n", "")
fmt:
	cargo fmt -- --config {{rustfmt_config}}
