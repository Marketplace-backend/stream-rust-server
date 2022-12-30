build:
	cargo build --release

run:
	cargo run --release

pre-commit:
	cargo build --release && cargo test --release && cargo fmt --all && cargo check --release

commit: pre-commit
	git add . && git commit -m $(argument) && git push