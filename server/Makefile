CARGOFLAGS :=
SERVICE := gcast-server.service
AUXSOURCES = $(SERVICE) config-default.toml
REMOTE_BUILD_EXCLUDE := --exclude /target/
CARGORELEASEFLAGS := --release

.PHONY: build
build:
	cargo build $(CARGOFLAGS)

include ../common.mk

ifneq ($(filter deploy%,$(MAKECMDGOALS)),)
include ../deploy-config.mk

.PHONY: deploy-build
deploy-build: build
deploy-build: CARGOFLAGS += --target $(DEPLOY_TARGET) $(CARGORELEASEFLAGS)

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
	$(RSYNC) $(AUXSOURCES) ../target/$(DEPLOY_TARGET)/release/server $(DEPLOY_SERVER_PATH)

.PHONY: deploy
deploy: deploy-build deploy-stop deploy-sync deploy-start

.PHONY: deploy-remote-sync
deploy-remote-sync:
	$(RSYNC) ../ $(REMOTE_BUILD_EXCLUDE) $(DEPLOY_BUILD_PATH)
	$(RSYNC) $(AUXSOURCES) $(DEPLOY_SERVER_PATH)

.PHONY: deploy-remote-build
deploy-remote-build:
	$(SSH) $(DEPLOY_HOST) cargo build --manifest-path $(DEPLOY_PATH)/build/Cargo.toml --bin server $(CARGORELEASEFLAGS)

.PHONY: deploy-remote-clean
deploy-remote-clean:
	$(SSH) $(DEPLOY_HOST) rm -rf $(DEPLOY_PATH)/build

.PHONY: deploy-remote-copy
deploy-remote-copy:
	$(SSH) $(DEPLOY_HOST) cp -f $(DEPLOY_PATH)/build/target/release/server $(DEPLOY_PATH)/server/server

.PHONY: deploy
deploy-remote: deploy-remote-sync deploy-remote-build deploy-stop deploy-remote-copy deploy-start
endif
