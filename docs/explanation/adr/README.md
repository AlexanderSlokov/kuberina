# Architecture Decision Records

An ADR records one decision, the situation that forced it, and what the project gave
up by taking it. It is written once and then left alone: when a later decision
overturns it, the old record is marked superseded rather than edited, because the
reason a choice looked right at the time is the part worth keeping.

These sit under `docs/explanation/` because they are understanding-oriented — they say
*why* the system is shaped the way it is. What the interfaces actually contain belongs
in `docs/references/`, and how to use them in `docs/how-to/`.

| ADR | Title | Status |
|---|---|---|
| [0001](0001-go-component-is-a-cli.md) | The Go component is a CLI, not an operator | Accepted |
| [0002](0002-kuberina-ir-v1-stability-contract.md) | Kuberina IR v1 stability contract | Accepted |

## Format

Each record carries: **Status**, **Context** (the forces in play, with evidence from
the repository), **Decision** (what is now true, stated in the present tense),
**Consequences** (what this makes easy, what it makes hard, what it forecloses), and
**Alternatives considered** with the reason each was rejected.

Number them sequentially. Never renumber.
