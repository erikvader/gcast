.PHONY: build
build:
	cargo build $(CARGOFLAGS)

ifneq ($(filter deploy,$(MAKECMDGOALS)),)
ifndef DEPLOY_TARGET
$(error DEPLOY_TARGET not defined while deploying)
endif

.PHONY: deploy
deploy: build
deploy: CARGOFLAGS += --target $(DEPLOY_TARGET)
endif
