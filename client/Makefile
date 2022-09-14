.PHONY: build
build:
	wasm-pack build --dev --target web --out-dir web-root/pkg --no-typescript .

.PHONY: serve
serve:
	@python serve.py

.PHONY: clean
clean:
	rm -rf web-root/pkg
