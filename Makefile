.PHONY: help install up down logs run dev test lint format check harness clean reset-db seed fresh
.DEFAULT_GOAL := help

PORT ?= 8000

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

install: ## Fetch dependencies and create .env if missing
	cargo fetch
	@test -f .env || cp .env.example .env

up: ## Start MongoDB and wait until healthy
	docker compose up -d --wait

down: ## Stop MongoDB (data volume is kept)
	docker compose down

logs: ## Tail MongoDB logs
	docker compose logs -f mongo

run: up ## Start MongoDB, then run the API with auto-reload
	PORT=$(PORT) cargo run

dev: run ## Alias for run

test: ## Run unit and integration tests (each run starts a throwaway MongoDB container)
	cargo test

lint: ## Run clippy (warnings are errors)
	cargo clippy --all-targets -- -D warnings

format: ## Format code and apply clippy fixes
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged --all-targets

check: ## Format check, lint, then tests (CI-style)
	cargo fmt --check
	$(MAKE) lint
	$(MAKE) test

harness: ## Open the browser test harness (API must be running)
	open http://localhost:$(PORT)/harness/

seed: up ## Load sample client scenarios (re-runnable)
	cargo run --bin seed

reset-db: up ## Drop the dev database (destructive)
	docker compose exec mongo mongosh --quiet --eval "db.getSiblingDB('liabilities').dropDatabase()"

fresh: ## Tear down Mongo and its volume, start it again, seed, then run the API (destructive)
	docker compose down -v
	$(MAKE) seed
	$(MAKE) run

clean: ## Remove build output
	cargo clean
