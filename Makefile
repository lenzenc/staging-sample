.PHONY: help install up down logs run dev test lint format check harness clean reset-db seed fresh
.DEFAULT_GOAL := help

PORT ?= 8000

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

install: ## Install dependencies and create .env if missing
	uv sync
	@test -f .env || cp .env.example .env

up: ## Start MongoDB and wait until healthy
	docker compose up -d --wait

down: ## Stop MongoDB (data volume is kept)
	docker compose down

logs: ## Tail MongoDB logs
	docker compose logs -f mongo

run: up ## Start MongoDB, then run the API with auto-reload
	uv run uvicorn app.main:app --reload --port $(PORT)

dev: run ## Alias for run

test: up ## Run the pytest suite (uses liabilities_test DB)
	uv run pytest -q

lint: ## Run ruff lint
	uv run ruff check .

format: ## Auto-fix lint issues and format code
	uv run ruff check --fix .
	uv run ruff format .

check: ## Lint, format check, then tests (CI-style)
	uv run ruff check .
	uv run ruff format --check .
	$(MAKE) test

harness: ## Open the browser test harness (API must be running)
	open http://localhost:$(PORT)/harness/

seed: up ## Load sample client scenarios (re-runnable)
	uv run python -m app.seed

reset-db: up ## Drop the dev database (destructive)
	docker compose exec mongo mongosh --quiet --eval "db.getSiblingDB('liabilities').dropDatabase()"

fresh: ## Tear down Mongo and its volume, start it again, seed, then run the API (destructive)
	docker compose down -v
	$(MAKE) seed
	$(MAKE) run

clean: ## Remove caches
	rm -rf .pytest_cache .ruff_cache
	find . -name __pycache__ -type d -not -path "./.venv/*" -prune -exec rm -rf {} +
