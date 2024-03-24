include ../common.mk

RELEASE :=
SERVICE := gcast-client.service
SOURCES := serve.py web-root

INDEXHTML := web-root/index.html
DEPSFOLDER := web-root/deps
3PP := normalize.css material-icons LeckerliOne-Regular.ttf
DEPS := $(3PP:%=$(DEPSFOLDER)/%)

.PHONY: all
all: build $(DEPSFOLDER) $(DEPS) $(INDEXHTML)

.PHONY: build
build:
	wasm-pack build $(if $(RELEASE),--release,--dev) --target web --out-dir web-root/pkg --no-typescript .

.PHONY: serve
serve:
	@python serve.py

.PHONY: devel
devel: build serve

$(DEPSFOLDER):
	mkdir -p $@

$(DEPSFOLDER)/normalize.css:
	$(CURL) -o $@ 'https://raw.githubusercontent.com/necolas/normalize.css/8.0.1/normalize.css'

$(DEPSFOLDER)/material-icons:
	$(CURL) -o $(DEPSFOLDER)/mi-tmp.tar.gz 'https://github.com/marella/material-icons/archive/refs/tags/v1.12.1.tar.gz'
	tar -C $(DEPSFOLDER) -xf $(DEPSFOLDER)/mi-tmp.tar.gz material-icons-1.12.1/iconfont
	mv $(DEPSFOLDER)/material-icons-1.12.1/iconfont $(DEPSFOLDER)/material-icons
	rm $(DEPSFOLDER)/material-icons/*.scss
	rmdir $(DEPSFOLDER)/material-icons-1.12.1
	rm $(DEPSFOLDER)/mi-tmp.tar.gz

$(DEPSFOLDER)/LeckerliOne-Regular.ttf:
	$(CURL) 'https://fonts.google.com/download?family=Leckerli%20One' | bsdtar -C $(DEPSFOLDER) -xf - $(@F)

# NOTE: PHONY to make sure it always gets the correct title
.PHONY: $(INDEXHTML)
$(INDEXHTML): index.html.in
	env IN_TITLE='gcast$(if $(RELEASE),, debug)' envsubst '$$IN_TITLE' < $< > $@

.PHONY: clean
clean:
	rm -rf web-root/pkg web-root/deps $(SERVICE)

ifneq ($(filter deploy%,$(MAKECMDGOALS)),)
include ../deploy-config.mk

$(SERVICE): $(SERVICE).in
	env IN_DEPLOY_PATH='$(DEPLOY_PATH)' envsubst '$$IN_DEPLOY_PATH' < $< > $@

.PHONY: deploy-build
deploy-build: all

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
	$(RSYNC) $^ $(SOURCES) $(DEPLOY_HOST)':$(DEPLOY_PATH)'/client/

.PHONY: deploy
deploy: deploy-build deploy-stop deploy-sync deploy-start
endif
