.PHONY: build
build:
	wasm-pack build --dev --target web --out-dir web-root/pkg --no-typescript . $(CARGOFLAGS)

.PHONY: serve
serve:
	@python serve.py

.PHONY: clean
clean:
	rm -rf web-root/pkg

ifneq ($(filter deploy,$(MAKECMDGOALS)),)
.PHONY: deploy
deploy: build
endif
