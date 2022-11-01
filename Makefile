# export CARGOFLAGS :=

#TODO: keep this makefile?

.PHONY: all
all: server client cli

.PHONY: server
server:
	$(MAKE) -C server build

.PHONY: client
client:
	$(MAKE) -C client build

.PHONY: cli
cli:
	cargo build -p cli $(CARGOFLAGS)

# ifneq ($(filter deploy deploy-server deploy-client,$(MAKECMDGOALS)),)
# include deploy-config.mk

# CARGOFLAGS += --release

# .PHONY: deploy deploy-server deploy-client
# deploy: deploy-server deploy-client

# deploy-client:
# 	$(MAKE) -C client deploy

# deploy-server:
# 	$(MAKE) -C server deploy
# endif
