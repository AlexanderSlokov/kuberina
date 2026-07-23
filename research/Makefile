immediate_run:
	uv run python -m \
	kuberina plan \
	--infra testdata/homelab_infra.yaml \
	--workloads testdata/homelab_workloads.yaml

# MSC Irina Scale stress test: 150 nodes, 2714 pods, 90% fill, hostile constraints
# GA: population=1024, max_gens=1000, early_stop=200
irina_stress:
	uv run python -m \
	kuberina plan \
	--infra testdata/irina_infra.yaml \
	--workloads testdata/irina_workloads.yaml

