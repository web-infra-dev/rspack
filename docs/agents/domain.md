# Domain Docs

This repo uses a single-context domain docs layout.

Before exploring, engineering skills should read these when present:

- `CONTEXT.md` at the repo root
- `docs/adr/` for architectural decisions that touch the area being changed

If these files do not exist, proceed silently. The domain-modeling flow can create them lazily when terms or decisions are resolved.

When output names a domain concept, use the vocabulary from `CONTEXT.md`. If a needed concept is missing, note it as a possible gap for domain modeling.

If output contradicts an existing ADR, surface that conflict explicitly.
