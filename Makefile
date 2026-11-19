# markers — Rust static site generator
# Common tasks: make help

CARGO ?= cargo
MARKERS ?= $(CARGO) run --release -p markers-cli --
PORT ?= 8080

.PHONY: help build release run serve docs test lint fmt fmt-check clippy \
        ci examples clean install landing

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  %-12s %s\n", $$1, $$2}'

build: ## Debug build (workspace)
	$(CARGO) build

release: ## Release build of the markers CLI
	$(CARGO) build --release -p markers-cli

run: release ## Run markers CLI: make run -- build docs --output dist
	./target/release/markers $(ARGS)

serve: release ## Serve the docs content with markers on :$(PORT)
	./target/release/markers serve packages/landing/docs --port $(PORT)

landing: ## Build the landing site (Next.js static export) → packages/landing/out
	pnpm -C packages/landing install --frozen-lockfile
	pnpm -C packages/landing build

docs: landing ## Alias for landing

test: ## Run workspace tests
	$(CARGO) test --workspace

lint: clippy ## Alias for clippy

clippy: ## Clippy with warnings denied
	$(CARGO) clippy --workspace --all-targets -- -D warnings

fmt: ## Format all Rust code
	$(CARGO) fmt

fmt-check: ## Verify formatting (CI check)
	$(CARGO) fmt --check

ci: ## Reproduce the CI flow locally (./scripts/ci.sh)
	./scripts/ci.sh

ci-deploy: ## CI flow + stage .page and build _site (like a master push)
	./scripts/ci.sh --deploy

examples: ## Wikipedia crawl + demo site (see scripts/examples.sh --help)
	./scripts/examples.sh $(ARGS)

install: release ## Install markers to ~/.local/bin
	cp target/release/markers $${HOME}/.local/bin/markers

clean: ## Remove build outputs and generated sites
	$(CARGO) clean
	rm -rf dist _site .page examples/demo
