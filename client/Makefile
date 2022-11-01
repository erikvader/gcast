RELEASE :=
SERVICE := gcast-client.service
SOURCES := serve.py web-root $(SERVICE)

.PHONY: build
build:
	wasm-pack build $(if $(RELEASE),--release,--dev) --target web --out-dir web-root/pkg --no-typescript .

.PHONY: serve
serve:
	@python serve.py

.PHONY: clean
clean:
	rm -rf web-root/pkg

include ../common.mk

ifneq ($(filter deploy%,$(MAKECMDGOALS)),)
include ../deploy-config.mk

.PHONY: deploy-build
deploy-build: build
deploy-build: RELEASE := t

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
	$(RSYNC) $(SOURCES) $(DEPLOY_CLIENT_PATH)

.PHONY: deploy
deploy: deploy-build deploy-stop deploy-sync deploy-start
endif
