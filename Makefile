fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

smoke:
	bash scripts/smoke_v0_2_0.sh

release-check: fmt lint test smoke

run:
	cargo run -p fa-server
