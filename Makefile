.PHONY: help install up down logs run dev test lint format check harness clean reset-db seed fresh
.DEFAULT_GOAL := help

PORT ?= 8000

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

install: ## Create .env if missing and download dependencies
	@test -f .env || cp .env.example .env
	./gradlew --console=plain classes testClasses

up: ## Start MongoDB and wait until healthy
	docker compose up -d --wait

down: ## Stop MongoDB (data volume is kept)
	docker compose down

logs: ## Tail MongoDB logs
	docker compose logs -f mongo

run: up ## Start MongoDB, then run the API (devtools restarts on rebuild)
	./gradlew bootRun --args='--server.port=$(PORT)'

dev: run ## Alias for run

test: ## Run the test suite (Testcontainers starts its own MongoDB; needs Docker)
	./gradlew test

lint: ## Check formatting and compile with -Xlint
	./gradlew spotlessCheck compileJava compileTestJava

format: ## Auto-format the code
	./gradlew spotlessApply

check: ## Format check, compile, then tests (CI-style)
	./gradlew check

harness: ## Open the browser test harness (API must be running)
	open http://localhost:$(PORT)/harness/

seed: up ## Load sample client scenarios (re-runnable)
	./gradlew bootRun --args='--spring.profiles.active=seed'

reset-db: up ## Drop the dev database (destructive)
	docker compose exec mongo mongosh --quiet --eval "db.getSiblingDB('liabilities').dropDatabase()"

fresh: ## Tear down Mongo and its volume, start it again, seed, then run the API (destructive)
	docker compose down -v
	$(MAKE) seed
	$(MAKE) run

clean: ## Remove build output
	./gradlew clean
