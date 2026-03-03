.PHONY: help docs-install docs-serve docs-build docs-clean build release test clean

help:
	@echo "vex - HTTP/3 Load Testing Tool"
	@echo ""
	@echo "Build & Release:"
	@echo "  make build          Build the project"
	@echo "  make release        Build release binary"
	@echo "  make test           Run tests"
	@echo "  make clean          Clean build artifacts"
	@echo ""
	@echo "Documentation (uses virtual environment):"
	@echo "  make docs-install   Install mkdocs dependencies in venv"
	@echo "  make docs-serve     Serve docs locally (http://localhost:8000)"
	@echo "  make docs-build     Build static documentation"
	@echo "  make docs-clean     Remove docs build artifacts"
	@echo ""
	@echo "Cleanup:"
	@echo "  make venv-clean     Remove virtual environment"
	@echo ""

# Build targets
build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean

# Virtual environment
VENV := .venv
PYTHON := $(VENV)/bin/python
PIP := $(VENV)/bin/pip
MKDOCS := $(VENV)/bin/mkdocs

$(VENV):
	@echo "Creating virtual environment..."
	python3 -m venv $(VENV)
	$(PIP) install --upgrade pip

# Documentation targets
docs-install: $(VENV)
	@echo "Installing mkdocs and dependencies..."
	$(PIP) install -r docs-requirements.txt
	@echo "Done! Run 'make docs-serve' to start the documentation server."

docs-serve: $(VENV)
	@echo "Starting mkdocs server..."
	@echo "Visit http://localhost:8000 in your browser"
	$(MKDOCS) serve

docs-build: $(VENV)
	@echo "Building static site..."
	$(MKDOCS) build
	@echo "Built documentation in site/ directory"

docs-clean:
	@echo "Cleaning docs build artifacts..."
	rm -rf site/
	rm -rf .mkdocs_cache/
	@echo "Done!"

venv-clean:
	@echo "Removing virtual environment..."
	rm -rf $(VENV)
	@echo "Done!"
