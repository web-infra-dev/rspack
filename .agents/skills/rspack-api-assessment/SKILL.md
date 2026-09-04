---
name: rspack-api-assessment
description: Strictly evaluate proposed or newly added public APIs, including hooks, specifically in web-infra-dev/rspack from Rspack's architecture, existing extension points, performance, webpack compatibility, and observed usage. Use when deciding whether Rspack should add or keep such a surface; default against addition when a viable Rspack substitute exists unless broad independent plugin compatibility is demonstrated.
---

# Assess an Rspack API

## Scope and outcome

Decide whether Rspack should add or keep a proposed API, including a hook, from Rspack's own architecture. Accept a PR, issue, commit, symbol, or design proposal and answer:

1. What does it do inside Rspack?
2. Which concrete scenario needs it?
3. Can existing Rspack capabilities replace it, and how?
4. What is its performance impact?
5. What is its architectural impact?
6. How is the analogous webpack API used?
7. Should Rspack add or keep it?

The final verdict must always include its reasons. State why the user and compatibility value does or does not outweigh the best substitute, performance cost, and long-term architectural commitment; never return only “add” or “do not add”.

Apply a strict public-API budget: the proposal carries the burden of proving that a permanent extension surface is necessary. If an existing Rspack solution can satisfy the required behavior correctly, reject the new API by default even when the substitute is later, less convenient, or requires rewriting one or a few plugins. Make an exception only when verified ecosystem evidence shows compatibility gains for a substantial set of independently maintained and actively used plugins; theoretical webpack parity, one consumer, forks, and copied implementations are insufficient.

This is read-only unless the user separately requests implementation or publication. Do not post comments or mutate GitHub state as part of the assessment.

## Gather evidence

Follow the repository `AGENTS.md`. Inspect the base and changed code, linked issue, public types, tests, documentation, CI, benchmarks, and relevant history instead of accepting the PR description as fact.

Read only the applicable Rspack guides:

- `.agents/API_DESIGN.md` for public APIs and compatibility contracts.
- `.agents/BINDING.md` for Rust/JavaScript crossings.
- `.agents/CACHE_AND_INCREMENTAL.md` for hashing, caching, watch, or incremental behavior.
- `.agents/ARCHITECTURE.md` for new compilation or rendering boundaries.

Prefer local source and Git history for Rspack, authenticated `gh` for GitHub evidence, and current webpack source for upstream comparisons. Cite stable source links.

## Evaluate from Rspack's architecture

### Function and scenario

Trace the exact call path and neighboring phases. Identify the owning subsystem, hook kind, data and return shape, lifetime, call frequency, and every renderer, runtime mode, output mode, or backend that must invoke it. Use precise boundaries: module source, rendered chunk body, and final emitted asset are different stages, and a render hook can still run after hashing.

Find a concrete consumer or reproducible workflow when possible. Explain the user-visible problem first and generalize only as far as the evidence supports.

### Existing Rspack substitutes

Construct the smallest credible alternative from the base version, considering module/parser hooks, runtime modules, render hooks, `processAssets`, `updateAsset`, built-in plugins, and small compositions.

Judge replacement separately at three levels:

- **Functional:** can it produce the same observable result?
- **Semantic:** does it preserve phase, scope, ordering, source maps, cache behavior, and lifecycle?
- **Compatibility:** can the webpack plugin or user code work without being rewritten?

Do not equate “later”, “awkward”, or “requires adaptation” with “impossible”; verify claims such as required reparsing against Rspack's actual Source APIs.

### Performance

Always separate:

- **Unused path:** empty hook calls, registration, allocations, locks, and whether JavaScript callbacks and binding conversion are skipped.
- **Active path:** call frequency multiplied by payload and per-call work, including source flattening, cloning, source-map generation, serialization, Rust/JavaScript transfer, synchronization, and returned-source reconstruction.

Check whether filtering occurs before expensive conversion, whether pass-through taps still transfer data, and whether callbacks serialize Rspack's parallel work or increase peak memory. Trace `CreateHash`, rendering, chunk render cache, `processAssets`, and incremental invalidation; determine how hook-controlled output enters `chunkHash`, `contentHash`, or cache keys.

Treat benchmark evidence narrowly: distinguish no tap, pass-through, and mutation, and never use untapped results to claim active use is free.

### Architecture and webpack compatibility

Judge whether the extension belongs in the Rspack subsystem that owns the phase and whether a narrower abstraction would better preserve the Rust-first architecture. Inspect:

- new or enlarged Rust/JavaScript boundaries;
- ordering guarantees and duplicated call sites that can drift across renderers;
- context identity, ownership, compilation lifetime, and cleanup;
- determinism, hash/cache correctness, incremental behavior, and concurrency boundaries;
- source maps, ESM/CommonJS syntax, target differences, errors, tests, and documentation.

For a webpack-compatible API, compare current webpack ordering, hook type, complete context shape, version, and behavior. A matching name is not full compatibility: classify missing fields as available through `compilation`, intentionally unsupported, or unavailable, and treat direct shape differences as compatibility debt.

### webpack and ecosystem usage

For a public compatibility API, search current public GitHub code using exact calls plus likely syntax variants. Exclude definitions, docs, tests, generated files, vendored stores, committed dependencies, source mirrors, datasets, and unrelated symbols; group copied implementations and forks into one independent usage pattern.

Report external repositories, independent patterns, notable downstream reach, search date, and the limits of public indexed code. When a viable Rspack substitute exists, compatibility can override the default rejection only if many independent, maintained plugins are demonstrably blocked or would work unchanged; one or several consumers, their forks, or downstream copies do not meet this bar. For an internal Rspack hook, inspect in-repository consumers and mark public usage not applicable.

## Make and explain the decision

Choose one outcome:

- **Add or keep:** no viable existing Rspack solution can satisfy a necessary scenario, or broad evidence shows that many independent plugins require this exact compatibility surface; the contract must also be narrow and its unused and active costs acceptable.
- **Add or keep with conditions:** the necessity or broad compatibility threshold is already met, but performance, context shape, hash/cache correctness, tests, or documentation needs a bounded follow-up; conditions cannot compensate for missing necessity.
- **Defer:** a specific missing fact about necessity, substitute viability, broad plugin usage, or active cost could materially change the decision.
- **Do not add:** use this as the default when a viable Rspack substitute exists and broad ecosystem compatibility has not been demonstrated, or when hot-path and architectural costs outweigh a genuinely necessary capability.

Determine the strongest reason for and against the API internally, but report only the decisive reason and any uncertainty that changes the verdict. The verdict must explicitly account for substitute viability and whether the broad-plugin compatibility threshold is met.

## Present the result

Return only the following seven labeled lines, with exactly one concise sentence per line and no headings, preamble, recap, table, or separate source list; place any necessary citation inline:

```text
Function: <what the API does>
Use case: <the concrete problem it solves>
Replaceability: <whether existing Rspack APIs or hooks can replace it, including the key difference>
Performance impact: <the main unused-path and active-path costs>
Architectural impact: <the most important architectural benefit or cost>
webpack usage: <the number and breadth of independent real-world usages>
Verdict: <add, keep, defer, or reject, with the decisive reason>
```

Only when `Replaceability` is an evidence-backed “yes” and existing Rspack APIs or hooks preserve the required behavior, append `Compatibility implementation:` followed by one minimal code block at the very end. Keep the snippet to the essential calls, normally no more than 12 non-empty lines; for partial, uncertain, or semantically different alternatives, output no code.
