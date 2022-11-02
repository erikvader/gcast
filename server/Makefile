CARGOFLAGS :=
SERVICE := gcast-server.service
SOURCES = ../target/$(DEPLOY_TARGET)/release/server $(SERVICE) config-default.toml

.PHONY: build
build:
	cargo build $(CARGOFLAGS)

include ../common.mk

ifneq ($(filter deploy%,$(MAKECMDGOALS)),)
include ../deploy-config.mk

.PHONY: deploy-build
deploy-build: build
deploy-build: CARGOFLAGS += --target $(DEPLOY_TARGET) --release

.PHONY: deploy-stop
deploy-stop:
	$(SSH) $(DEPLOY_HOST) $(SERVICESTOP) $(SERVICE)

.PHONY: deploy-start
deploy-start:
	$(SSH) $(DEPLOY_HOST) $(SERVICESTART) $(SERVICE)

.PHONY: deploy-status
deploy-status:
	$(SSH) $(DEPLOY_HOST) $(SERVICESTATUS) $(SERVICE)

.PHONY: deploy-sync
deploy-sync:
	$(RSYNC) $(SOURCES) $(DEPLOY_SERVER_PATH)

.PHONY: deploy
deploy: deploy-build deploy-stop deploy-sync deploy-start
endif
