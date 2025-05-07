# SPDX-FileCopyrightText: Copyright (c) 2022-2025 Yegor Bugayenko
# SPDX-License-Identifier: MIT

DOTS := $(wildcard *.dot)
BINS := $(patsubst %.dot,%.pdf,$(DOTS))
PDF = doc.pdf

.ONESHELL:
.SHELLFLAGS: -e -o pipefail -c
.PHONY: clean
SHELL = bash

$(PDF): $(BINS)
	pdflatex -interaction=batchmode -halt-on-error -shell-escape doc.tex

%.pdf: %.dot
	dot -Tpdf -o $@ $<

clean:
	rm -rf $(BINS)
	rm -rf $(PDF)
