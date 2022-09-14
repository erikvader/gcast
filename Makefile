.PHONY: server
server:
	cargo build -p server

.PHONY: client
client:
	cargo build -p client --target wasm32-unknown-unknown

