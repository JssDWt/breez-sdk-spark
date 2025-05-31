SUBDIRS := $(wildcard */.)

.PHONY: $(SUBDIRS)

$(SUBDIRS):
	$(MAKE) -C $@ $(MAKECMDGOALS)

default: check

build: $(SUBDIRS)

build-release: $(SUBDIRS)

build-wasm: $(SUBDIRS)

check: fmt-check clippy-check test

clippy: clippy-check

clippy-fix: $(SUBDIRS)

clippy-check: $(SUBDIRS)

fix: fmt-fix clippy-fix

fmt: fmt-fix

fmt-fix: $(SUBDIRS)

fmt-check: $(SUBDIRS)

test: $(SUBDIRS)

wasm-clippy-check: $(SUBDIRS)
