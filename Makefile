.ONESHELL:
.SHELLFLAGS: -e -x -o pipefail -c
.PHONY: clean

SHELL = bash
REO = target/debug/reo --verbose

SODGS = $(shell find target/eo/sodg -type f -name '*.sodg')
BINARIES = $(subst .sodg,.reo,$(subst sodg/,reo/,$(SODGS)))

target/runtime.reo: ${BINARIES}
	rm -f $@
	$(REO) empty $@
	for b in $(BINARIES); do $(REO) merge $@ $${b}; done

target/eo/1-parse:
	mvn -C test-pom.xml

$(BINARIES): target/eo/1-parse
	mkdir -p $$(dirname $@)
	$(REO) compile $(subst .reo,.sodg,$(subst eo/reo/,eo/sodg/,$@)) $@

clean:
	rm -rf $(BINARIES)
	rm -rf target/runtime.reo