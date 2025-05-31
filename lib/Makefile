SUBDIRS := $(wildcard */.)

.PHONY: $(SUBDIRS)

$(SUBDIRS):
	$(MAKE) -C $@ $(MAKECMDGOALS)

default: check

bindings-csharp: 
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-go: 
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-kotlin-multiplatform:
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-kotlin:
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-python:
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-swift:
	$(MAKE) -C bindings $(MAKECMDGOALS)

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
