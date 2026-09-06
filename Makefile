# Kuberina — component targets.
#
# Four components, four target prefixes:
#   forge-*      Go frontend and linker, manifests <-> IR (forge/)
#   solver-*     Rust optimization engine (solver/)
#   inspector-*  Independent constraint validator + heatmap (inspector/)
#   bench-*      Testdata generation and formal proofs (bench/)
#   research-*   Python reference implementation of the 3-phase pipeline (research/)

# --- Forge (Go) ------------------------------------------------------------

# The only component that speaks Kubernetes. A CLI, never a controller: ADR-0001.
forge-build:
	go build -o bin/kuberina-forge ./forge/cmd/kuberina-forge

forge-test:
	go test ./forge/...

forge-lint:
	gofmt -l forge && go vet ./forge/...

# End-to-end: manifests -> IR -> blueprint -> pinned manifests.
forge-demo: forge-build solver-build
	./bin/kuberina-forge in \
		--out-infra /tmp/kuberina-demo-infra.yaml \
		--out-workloads /tmp/kuberina-demo-workloads.yaml \
		forge/testdata/manifests
	cd solver && cargo run --release -- plan \
		--infra /tmp/kuberina-demo-infra.yaml \
		--workloads /tmp/kuberina-demo-workloads.yaml
	./bin/kuberina-forge out \
		--blueprint solver/kuberina_solution.yaml \
		--out /tmp/kuberina-demo-planned \
		forge/testdata/manifests

# --- Solver (Rust) ---------------------------------------------------------

solver-build:
	cd solver && cargo build

solver-test:
	cd solver && cargo test

solver-clippy:
	cd solver && cargo clippy

solver-lint:
	cd solver && cargo clippy -- -D warnings && cargo fmt --check

solver-fmt:
	cd solver && cargo fmt

solver-homelab:
	cd solver && cargo run --release -- plan \
	--infra testdata/homelab_infra.yaml \
	--workloads testdata/homelab_workloads.yaml

solver-irina:
	cd solver && cargo run --release -- plan \
	--infra testdata/irina_infra.yaml \
	--workloads testdata/irina_workloads.yaml

solver-irina-headroom-20:
	cd solver && cargo run --release -- plan \
	--infra testdata/irina_infra.yaml \
	--workloads testdata/irina_workloads.yaml \
	--headroom 20

# --- Inspector (independent validator) -------------------------------------

# Re-reads infra/workloads/solution from scratch and re-checks every constraint.
# Shares no code with the solver — that independence is the point.
inspector-run:
	uv run --with pyyaml python inspector/inspector.py \
		--infra solver/testdata/irina_infra.yaml \
		--workloads solver/testdata/irina_workloads.yaml \
		--solution solver/kuberina_solution.yaml \
		--output kuberina_dashboard.html

# --- Benchmarks ------------------------------------------------------------

# Writes into solver/testdata (cwd-relative output inside the generator).
bench-generate-testdata:
	cd solver && uv run --with pyyaml python ../bench/gen_irina_testdata.py

bench-proof:
	uv run --with pyyaml python bench/mathematical_proof.py

# The reserve must match the one the solver was given, or the optimality bound is
# computed against a cluster the optimizer never saw. See #8.
bench-proof-headroom-20:
	uv run --with pyyaml python bench/mathematical_proof.py --headroom 20

# --- Research (Python reference implementation) -----------------------------

research-homelab:
	cd research && $(MAKE) immediate_run

research-irina:
	cd research && $(MAKE) irina_stress

# Writes into research/testdata so the reference implementation runs on its own copy.
research-generate-testdata:
	cd research && uv run --with pyyaml python ../bench/gen_irina_testdata.py

# --- Pipelines -------------------------------------------------------------

full-pipeline: ## Generate testdata -> solve -> inspect -> prove
	@echo "=> Generating 8D testdata..."
	cd research && uv run python ../bench/gen_irina_testdata.py
	@echo "=> Running solver on generated testdata..."
	cd solver && cargo run --release -- plan \
		--infra ../research/testdata/irina_infra.yaml \
		--workloads ../research/testdata/irina_workloads.yaml \
		--headroom 20
	@echo "=> Running inspector heatmap & validation..."
	uv run --with pyyaml python inspector/inspector.py \
		--infra research/testdata/irina_infra.yaml \
		--workloads research/testdata/irina_workloads.yaml \
		--solution solver/kuberina_solution.yaml
	@echo "=> Running formal mathematical proof..."
	uv run --with pyyaml python bench/mathematical_proof.py
	@echo "=> Pipeline complete."

.PHONY: forge-build forge-test forge-lint forge-demo \
	solver-build solver-test solver-clippy solver-lint solver-fmt \
	solver-homelab solver-irina solver-irina-headroom-20 \
	inspector-run bench-generate-testdata bench-proof bench-proof-headroom-20 \
	research-homelab research-irina research-generate-testdata full-pipeline
