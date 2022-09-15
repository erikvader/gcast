.PHONY: all
all: server client

.PHONY: server
server:
	cargo build -p server

.PHONY: client
client:
	$(MAKE) -C client build

