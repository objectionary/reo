# SPDX-FileCopyrightText: Copyright (c) 2022-2026 Yegor Bugayenko
# SPDX-License-Identifier: MIT

# The objects from eo-tests/, together with the objects of the EO runtime
# they use, are compiled to XMIR by eo-maven-plugin, which also names their
# positional arguments after the voids they land in (the dealpha goal,
# which needs the infer goal before it), then to .sodg files by `phino
# sodg`, then to binary .reo files by REO, and then merged into
# target/runtime.reo, which tests/eo_test.rs dataizes.

.ONESHELL:
.SHELLFLAGS := -e -o pipefail -c
.PHONY: all clean test
.SECONDARY:

SHELL = bash
REO = target/debug/reo
PHINO = phino

XMIR = $(firstword $(wildcard target/eo/*-dealpha))
XMIRS = $(shell find $(XMIR) -type f -name '*.xmir' 2>/dev/null)
SODGS = $(patsubst $(XMIR)/%.xmir,target/eo/sodg/%.sodg,$(XMIRS))
BINARIES = $(patsubst target/eo/sodg/%.sodg,target/eo/reo/%.reo,$(SODGS))

all: target/eo-foreign.csv
	$(MAKE) target/runtime.reo

test:
	cargo test

target/eo-foreign.csv: test-pom.xml $(wildcard eo-tests/*.eo)
	mvn --file test-pom.xml --batch-mode --errors process-classes
	touch $@

target/runtime.reo: $(BINARIES) $(REO)
	rm -f $@
	$(REO) empty $@
	for b in $(BINARIES); do $(REO) merge $@ $${b}; done

$(REO):
	cargo build

target/eo/sodg/%.sodg: $(XMIR)/%.xmir
	mkdir -p $(@D)
	$(PHINO) sodg --input xmir $< > $@.tmp
	mv $@.tmp $@

target/eo/reo/%.reo: target/eo/sodg/%.sodg $(REO)
	mkdir -p $(@D)
	$(REO) compile $< $@

clean:
	rm -rf target/eo
	rm -f target/runtime.reo
	rm -f target/eo-foreign.csv
