default: release

prepare:
	rustup target  add x86_64-unknown-linux-musl

release: prepare
	cargo build --release --target=x86_64-unknown-linux-musl
