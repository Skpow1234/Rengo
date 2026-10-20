# Rengo Development Commands
.PHONY: help format check test clean docker-build docker-up docker-down docker-dev docker-perf docker-cluster

help: ## Show this help message
	@echo "Rengo Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

format: ## Format code with cargo fmt
	@echo "🔧 Formatting code..."
	cargo fmt
	@echo "✅ Code formatted!"

check: ## Run all checks (format, clippy, test)
	@echo "🔍 Running all checks..."
	cargo fmt --check
	cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::cargo
	cargo test --workspace
	@echo "✅ All checks passed!"

check-warnings: ## Run checks allowing warnings (for CI/CD)
	@echo "🔍 Running checks with warnings allowed..."
	cargo fmt --check
	cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::cargo || echo "⚠️ Clippy warnings found but continuing..."
	cargo test --workspace
	@echo "✅ Checks completed (warnings allowed)!"

test: ## Run tests
	@echo "🧪 Running tests..."
	cargo test --workspace --verbose

test-integration: ## Run integration tests
	@echo "🧪 Running integration tests..."
	cargo test --test integration
	cargo test --test storage_integration
	cargo test --test config_integration

bench: ## Run benchmarks
	@echo "⚡ Running benchmarks..."
	cargo bench --workspace

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	cargo clean

install-dev: ## Install development dependencies
	@echo "📦 Installing development tools..."
	cargo install cargo-audit
	cargo install cargo-tarpaulin

audit: ## Run security audit
	@echo "🔒 Running security audit..."
	cargo audit

coverage: ## Generate coverage report
	@echo "📊 Generating coverage report..."
	cargo tarpaulin --workspace --out Html --output-dir coverage

# Docker commands
docker-build: ## Build Docker image
	@echo "🐳 Building Docker image..."
	docker build -t rengo:alpine-secure .

docker-up: ## Start Rengo in production mode
	@echo "🚀 Starting Rengo..."
	docker-compose --profile production up -d

docker-down: ## Stop all Rengo services
	@echo "🛑 Stopping Rengo..."
	docker-compose down

docker-dev: ## Start Rengo in development mode
	@echo "🔧 Starting Rengo in development mode..."
	docker-compose --profile dev up -d

docker-perf: ## Start Rengo in performance testing mode
	@echo "⚡ Starting Rengo in performance mode..."
	docker-compose --profile perf up -d

docker-cluster: ## Start Rengo cluster (3 nodes)
	@echo "🌐 Starting Rengo cluster..."
	docker-compose --profile cluster up -d

docker-monitoring: ## Start monitoring stack (Prometheus + Grafana)
	@echo "📊 Starting monitoring stack..."
	docker-compose --profile monitoring up -d

docker-logs: ## Show logs for all services
	@echo "📋 Showing logs..."
	docker-compose logs -f

docker-clean: ## Clean up Docker resources
	@echo "🧹 Cleaning Docker resources..."
	docker-compose down -v
	docker system prune -f

docker-scan: ## Scan Docker image for vulnerabilities
	@echo "🔍 Scanning Docker image for vulnerabilities..."
	docker scout cves rengo:alpine-secure


docker-secure: ## Build and scan Docker image for security
	@echo "🔒 Building and scanning secure Docker image..."
	make docker-build
	make docker-scan

# Windows PowerShell compatibility
# To use on Windows, you can run: make check
# Or use: cargo make check (if you install cargo-make)
