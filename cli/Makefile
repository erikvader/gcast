include ../common.mk

RELEASE :=
CARGO_BUILDFLAGS += $(if $(RELEASE),--release,)

.PHONY: build
build:
	$(CARGO_BUILD)
