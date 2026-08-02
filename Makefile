# Solver targets (Rust)
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

solver-irina-pareto-80:
	cd solver && cargo run --release -- plan \
	--infra testdata/irina_infra.yaml \
	--workloads testdata/irina_workloads.yaml \
	--pareto 80

# Research targets (Python)
research-generate-testdata:
	cd solver && uv run --with pyyaml python ../research/gen_irina_testdata.py

research-homelab:
	cd research && $(MAKE) immediate_run

research-irina:
	cd research && $(MAKE) irina_stress

research-inspect:
	uv run --with pyyaml python research/inspector.py \
		--infra solver/testdata/irina_infra.yaml \
		--workloads solver/testdata/irina_workloads.yaml \
		--solution solver/kuberina_solution.yaml \
		--output kuberina_dashboard.html

research-verify:
	uv run --with pyyaml python research/mathematical_proof.py