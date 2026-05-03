---
dip: 0000
title: "Short imperative title — start with a verb if possible"
status: Draft
authors:
  - you@example.com
created: 2026-05-03
updated: 2026-05-03
discussion:
implementation:
supersedes:
superseded-by:
---

## Summary

One paragraph. What is changing, and what's the user-visible effect? A
reviewer should be able to read this and decide whether to keep reading.

## Motivation

Why does this need to happen now? What goes wrong if we don't do it? Cite
incidents, customer asks, or constraints — not aesthetics.

## Detailed design

The actual proposal. Be specific:

- New API surface (routes, CLI flags, config keys, SQL columns)
- Compatibility with existing surfaces — what breaks, what migrates
- Failure modes the design accounts for
- Anything a reviewer would otherwise have to infer from a diff

Code snippets, schemas, sequence diagrams welcome. Be concrete.

## Alternatives considered

What other approaches were on the table, and why this one? At least one real
alternative — "do nothing" doesn't count unless it's load-bearing.

## Migration & rollout

How does this land safely?
- What ships first vs. later?
- Is there a feature flag, dual-write window, or staged rollout?
- How do we revert if it goes wrong?

## Open questions

Anything intentionally left unresolved. These should shrink to zero before
the DIP moves to Accepted.

## Out of scope

Things people will reasonably ask about that this DIP explicitly does not
solve.
