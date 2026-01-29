.PHONY: bench

# Run benchmark for a specific crate
# Usage: make bench CRATE=engine [DESC=description]
bench:
ifndef CRATE
	@echo "Error: CRATE parameter required"
	@echo "Usage: make bench CRATE=<name> [DESC=<description>]"
	@echo "Example: make bench CRATE=engine DESC=baseline"
	@exit 1
endif
	@./scripts/run_benchmark.sh $(CRATE) $(DESC)
