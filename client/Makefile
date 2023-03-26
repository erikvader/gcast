RELEASE :=
SERVICE := gcast-client.service
SOURCES := serve.py web-root $(SERVICE)

INDEXHTML := web-root/index.html
DEPSFOLDER := web-root/deps
3PP := normalize.css material-icons LeckerliOne-Regular.ttf
DEPS := $(3PP:%=$(DEPSFOLDER)/%)
CURLFLAGS := -L

.PHONY: all
all: build $(DEPSFOLDER) $(DEPS) $(INDEXHTML)

.PHONY: build
build:
	wasm-pack build $(if $(RELEASE),--release,--dev) --target web --out-dir web-root/pkg --no-typescript .

.PHONY: serve
serve:
	@python serve.py

$(DEPSFOLDER):
	mkdir -p $@

$(DEPSFOLDER)/normalize.css:
	curl $(CURLFLAGS) -o $@ 'https://raw.githubusercontent.com/necolas/normalize.css/8.0.1/normalize.css'

$(DEPSFOLDER)/material-icons:
	curl $(CURLFLAGS) -o $(DEPSFOLDER)/mi-tmp.tar.gz 'https://github.com/marella/material-icons/archive/refs/tags/v1.12.1.tar.gz'
	tar -C $(DEPSFOLDER) -xf $(DEPSFOLDER)/mi-tmp.tar.gz material-icons-1.12.1/iconfont
	mv $(DEPSFOLDER)/material-icons-1.12.1/iconfont $(DEPSFOLDER)/material-icons
	rm $(DEPSFOLDER)/material-icons/*.scss
	rmdir $(DEPSFOLDER)/material-icons-1.12.1
	rm $(DEPSFOLDER)/mi-tmp.tar.gz

$(DEPSFOLDER)/LeckerliOne-Regular.ttf:
	curl $(CURLFLAGS) 'https://fonts.google.com/download?family=Leckerli%20One' | bsdtar -C $(DEPSFOLDER) -xf - $(@F)

# NOTE: PHONY to make sure it always gets the correct title
.PHONY: $(INDEXHTML)
$(INDEXHTML): index.html.in
	sed 's/###TITLE###/gcast$(if $(RELEASE),, debug)/' < $< > $@

.PHONY: clean
clean:
	rm -rf web-root/pkg web-root/deps

include ../common.mk

ifneq ($(filter deploy%,$(MAKECMDGOALS)),)
include ../deploy-config.mk

.PHONY: deploy-build
deploy-build: all
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
# NOTE: colon is included here because of annoying syntax highlighting in emacs
	$(RSYNC) $(SOURCES) $(DEPLOY_HOST)':$(DEPLOY_PATH)'/client/

.PHONY: deploy
deploy: deploy-build deploy-stop deploy-sync deploy-start
endif
