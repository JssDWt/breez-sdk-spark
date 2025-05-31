SUBDIRS := cli lib
WASM_SUBDIRS := lib

.PHONY: $(SUBDIRS)

$(SUBDIRS):
	$(MAKE) -C $@ $(MAKECMDGOALS)

default: check

build: $(SUBDIRS)

build-wasm: $(WASM_SUBDIRS)

build-release: $(SUBDIRS)

check: fmt-check clippy-check test

clippy-fix: $(SUBDIRS)

clippy-check: $(SUBDIRS)

fix: fmt-fix clippy-fix

fmt-fix: $(SUBDIRS)

fmt-check: $(SUBDIRS)

test: $(SUBDIRS)

wasm-clippy-check: $(WASM_SUBDIRS)