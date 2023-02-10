

lint:
	cargo fmt -- --check --color always
	cargo clippy --all-targets --all-features -- -D warnings

