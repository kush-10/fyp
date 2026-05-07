.DEFAULT_GOAL := all

PDFLATEX := pdflatex -interaction=nonstopmode -halt-on-error
BIBTEX := bibtex
LATEX_ARTIFACTS := *.aux *.log *.bbl *.blg *.out *.toc *.lof *.lot *.fdb_latexmk *.fls *.synctex.gz

.PHONY: default all build archive clean

default: all

all: build

build:
	$(PDFLATEX) fyp-report.tex
	$(BIBTEX) fyp-report
	$(PDFLATEX) fyp-report.tex
	$(PDFLATEX) fyp-report.tex
	$(PDFLATEX) fyp-report.tex

archive: build
	rm -f archive.zip
	zip -r archive.zip Figures/ Makefile *.cls *.tex *.bib fyp-report.pdf -x "*.svn*"

clean:
	rm -f fyp-report.pdf archive.zip .DS_Store $(LATEX_ARTIFACTS)
