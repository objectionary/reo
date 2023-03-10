.ONESHELL:
.SHELLFLAGS: -e -x -o pipefail -c
.PHONY: clean

SHELL = bash
REO = target/debug/reo --verbose

SODGS = $(shell find . -type f -path './target/eo/sodg/*' -name '*.sodg')
BINARIES = $(subst .sodg,.reo,$(subst sodg/,reo/,$(SODGS)))

target/runtime.reo: target/eo/1-parse ${BINARIES} $(REO)
	rm -f $@
	$(REO) empty $@
	for b in $(BINARIES); do $(REO) merge $@ $${b}; done

$(REO):
	cargo build -vv

target/eo/1-parse:
	mvn --file test-pom.xml --batch-mode --errors process-resources

$(BINARIES): target/eo/1-parse $(SODGS)
	mkdir -p $$(dirname $@)
	$(REO) compile $(subst .reo,.sodg,$(subst eo/reo/,eo/sodg/,$@)) $@

clean:
	rm -rf target/eo/sodg
	rm -rf target/runtime.reo
	rm -rf target/debug/reo
