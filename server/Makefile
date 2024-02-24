SERVICE := gcast-server.service
AUXSOURCES := config.def.toml

RELEASE :=
CARGO_BUILDFLAGS += $(if $(RELEASE),--release,)
TARGET_SUBDIR += $(if $(RELEASE),release,debug)

.PHONY: build
build:
	$(CARGO_BUILD)

.PHONY: clean
clean:
	rm -f $(SERVICE)

include ../common.mk

ifneq ($(filter deploy%,$(MAKECMDGOALS)),)
include ../deploy-config.mk

$(SERVICE): $(SERVICE).in
	env IN_DEPLOY_PATH='$(DEPLOY_PATH)' envsubst '$$IN_DEPLOY_PATH' < $< > $@

.PHONY: deploy-build
deploy-build: build
deploy-build: CARGO_BUILDFLAGS += --target $(DEPLOY_TARGET)

.PHONY: deploy-stop
deploy-stop:
	$(SSH) $(DEPLOY_HOST) $(SERVICE_STOP) $(SERVICE)

.PHONY: deploy-start
deploy-start:
	$(SSH) $(DEPLOY_HOST) $(SERVICE_START) $(SERVICE)

.PHONY: deploy-status
deploy-status:
	$(SSH) $(DEPLOY_HOST) $(SERVICE_STATUS) $(SERVICE)

.PHONY: deploy-journal
deploy-journal:
	$(SSH) $(DEPLOY_HOST) $(SERVICE_JOURNAL) $(SERVICE)

.PHONY: deploy-sync
deploy-sync: $(SERVICE)
# NOTE: colon is included here because of annoying syntax highlighting in emacs
	$(RSYNC) $^ $(AUXSOURCES) ../target/$(DEPLOY_TARGET)/$(TARGET_SUBDIR)/server $(DEPLOY_HOST)':$(DEPLOY_PATH)'/server/

.PHONY: deploy
deploy: deploy-build deploy-stop deploy-sync deploy-start

.PHONY: deploy-remote-sync
deploy-remote-sync: $(SERVICE)
# NOTE: colon is included here because of annoying syntax highlighting in emacs
	$(RSYNC) ../ --exclude /target/ $(DEPLOY_HOST)':$(DEPLOY_PATH)'/build/
	$(RSYNC) $^ $(AUXSOURCES) $(DEPLOY_HOST)':$(DEPLOY_PATH)'/server/

.PHONY: deploy-remote-build
deploy-remote-build: CARGO_BUILDFLAGS += --manifest-path '$(DEPLOY_PATH)'/build/Cargo.toml --bin server
deploy-remote-build:
	$(SSH) $(DEPLOY_HOST) $(CARGO_BUILD)

.PHONY: deploy-remote-clean
deploy-remote-clean:
	$(SSH) $(DEPLOY_HOST) rm -rf '$(DEPLOY_PATH)'/build

.PHONY: deploy-remote-copy
deploy-remote-copy:
	$(SSH) $(DEPLOY_HOST) cp -f '$(DEPLOY_PATH)'/build/target/$(TARGET_SUBDIR)/server '$(DEPLOY_PATH)'/server/server

.PHONY: deploy-remote
deploy-remote: deploy-remote-sync deploy-remote-build deploy-stop deploy-remote-copy deploy-start
endif
