# Solver targets (Rust)
solver-build:
	cd solver && cargo build

solver-test:
	cd solver && cargo test

solver-clippy:
	cd solver && cargo clippy

solver-homelab:
	cd solver && cargo run -- plan \
	--infra testdata/homelab_infra.yaml \
	--workloads testdata/homelab_workloads.yaml

solver-irina:
	cd solver && cargo run --release -- plan \
	--infra testdata/irina_infra.yaml \
	--workloads testdata/irina_workloads.yaml

# Research targets (Python)
research-homelab:
	cd research && $(MAKE) immediate_run

research-irina:
	cd research && $(MAKE) irina_stress
