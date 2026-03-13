fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

smoke:
	bash scripts/smoke_v0_2_0.sh

smoke-sandbox:
	bash scripts/smoke_v0_2_0_sandbox.sh

release-check: fmt lint test smoke

release-check-sandbox: fmt lint test smoke-sandbox

run:
	cargo run -p fa-server
