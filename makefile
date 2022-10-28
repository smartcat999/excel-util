

build-linux:
	cargo build --release --target=x86_64-unknown-linux-gnu 

build-windows:
	cargo build --release --target=x86_64-pc-windows-gnu

build-macos:
	cargo build --release --target=aarch64-apple-darwin

all: help

help:
	@echo "usage: make build-linux or make build-windows or make build-macos"