build:
	cargo build --release

run:
	cargo run --release

check:
	cargi check --release

pre-commit:
	cargo check --release && cargo build --release && cargo test --release && cargo clippy --release && cargo fmt --all

commit: pre-commit
	git add . && git commit -m "$(argument)" && git push