DOTS := $(wildcard *.dot)
BINS := $(patsubst %.dot,%.pdf,$(DOTS))
PDF = doc.pdf

.ONESHELL:
.SHELLFLAGS: -e -o pipefail -c
.PHONY: clean
SHELL = bash

$(PDF): $(BINS)
	pdflatex doc.tex

%.pdf: %.dot
	dot -Tpdf -o $@ $<

clean:
	rm -rf $(BINS)
	rm -rf $(PDF)