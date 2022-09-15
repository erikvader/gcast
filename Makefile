.PHONY: all
all: server client cli

.PHONY: server
server:
	cargo build -p server

.PHONY: client
client:
	$(MAKE) -C client build

.PHONY: cli
cli:
	cargo build -p cli
