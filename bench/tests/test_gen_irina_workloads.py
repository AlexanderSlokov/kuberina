"""Tests for the MSC Irina workloads generator: IR v1 shape and constraint placement."""

from gen_irina_testdata import (
    AFFINITY_CHAINS,
    ANTI_AFFINITY_CROSS,
    SERVICES,
    generate_workloads,
)


def declarations(workloads):
    """Map `namespace/name` to its declaration dict."""
    return {
        f"{ns}/{decl['name']}": decl
        for ns, decls in workloads["namespaces"].items()
        for decl in decls
    }


def key_of(service):
    """The `namespace/name` key of a service by its name."""
    namespace = next(s[1] for s in SERVICES if s[0] == service)
    return f"{namespace}/{service}"


WORKLOADS = generate_workloads()
DECLS = declarations(WORKLOADS)


def test_document_has_only_ir_v1_top_level_keys():
    # Regression for #26: a top-level `pods:` list held every constraint, and the
    # solver never read it.
    assert set(WORKLOADS) <= {"apiVersion", "kind", "namespaces", "groups"}


def test_only_spread_services_repel_their_own_replicas():
    # Regression: the spread filter read column 7 (disk read), which is non-zero for
    # every service, so all 49 services were spread instead of the flagged ones.
    for service in SERVICES:
        key = f"{service[1]}/{service[0]}"
        repels_itself = key in DECLS[key].get("antiAffinity", [])
        assert repels_itself == service[12], key


def test_cross_tier_pairs_are_declaration_level_anti_affinity():
    for svc_a, svc_b in ANTI_AFFINITY_CROSS:
        assert key_of(svc_b) in DECLS[key_of(svc_a)]["antiAffinity"]


def test_each_chain_link_is_an_affinity_rule_toward_the_next_service():
    for chain in AFFINITY_CHAINS:
        for src, dst in zip(chain, chain[1:]):
            assert key_of(dst) in DECLS[key_of(src)]["affinity"]


def test_every_target_names_a_declaration_in_the_document():
    for decl in DECLS.values():
        for target in decl.get("affinity", []) + decl.get("antiAffinity", []):
            assert target in DECLS, target
