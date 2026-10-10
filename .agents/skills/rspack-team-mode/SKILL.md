---
name: rspack-team-mode
description: Apply observed Rspack maintainer conventions when asked for Rspack team mode or to write or review in the Rspack core team's style.
---

# Rspack team mode

Apply observed conventions from public `web-infra-dev/rspack` activity during 2025-10-06 through 2026-10-06. This mode covers code writing and review. The [roster](references/roster.md) distinguishes 20 active owners, including one inferred specialist, from sparse and uncertain candidates. It does not speak for these people or grant their approval.

The evidence is a representative sample. A blinded, diff-only smoke test did not reproduce the selected maintainers' specific review requests; reviewer fidelity remains unverified. Read the relevant person's ledger before applying an attributed convention. Repository requirements apply across this project; personal conventions apply only where supported. Do not infer unanimity or turn a tentative suggestion into a merge blocker. See [scope](references/scope.md) and [validation](references/validation.md) for coverage and limits.

## Start with the repository contract

Follow the current [AGENTS.md](../../../AGENTS.md), including building consumed artifacts before tests, existing JavaScript case runners, dedicated Rust test crates, and English/Chinese documentation for public changes. Read [API design](../../API_DESIGN.md) for compatibility and versioning. Consult the subsystem references linked from AGENTS.md before changing cache, concurrency, bindings, or sources.

Use actual webpack behavior to settle a compatibility question, and state intentional differences. A documented performance tradeoff can justify a difference; this mode does not require identical implementation. Keep a public rename or stabilization compatible through the applicable deprecation path. Internal components that ship together may have different compatibility constraints.

## Route by the changed behavior

- Public options, defaults, exports, and documentation: use chenjiahan's contract and migration checks. For lifecycle or webpack semantic mismatches, use hardfist's evidence. Read [core A](references/core-a.md).
- Performance and benchmarks: use LingyuCoder's separation of benchmark timing from correctness tests and stormslowly's measured, selective optimizations. Preserve observable behavior while reducing allocation or synchronization cost. Read [core A](references/core-a.md).
- CSS, copying, environment replacement, and loader snapshots: use intellild's producer/consumer boundaries and focused edge-case regressions. For source-map construction, caching, or diagnostic source lifetimes, use SyMind's boundary-specific review expectations. Several supporting proposals are unmerged. Read [core A](references/core-a.md).
- Incremental and persistent cache: use ahabhgk's dependency/runtime/output-path invalidation checks and jerrykingxyz's separation of cache validity, storage policy, and downstream recovery. Read [core B](references/core-b.md).
- ESM, deferred imports, and library output: use JSerFeng's linking/evaluation checks. Choose runtime execution for evaluation timing and emitted content/path assertions for layout changes. For compiler integration and Rust mutation safety, consult CPunisher and quininer respectively. Read [core B](references/core-b.md).
- Module Federation manifests and fallback loading: use 2heal1's emitted-data and executable-fallback cases. For Rslint upgrades, fansenze's evidence covers config migration and intentional rule choices only. Read [core B](references/core-b.md).
- Rstest, Rslib, Rsdoctor, website tooling, Module Federation design, and watchers: consult the relevant accounts in [specialists](references/specialists.md). Sparse records provide ownership context, not a general review persona.

## Keep disagreements and exceptions visible

For source maps, SyMind distinguished parsing from fetch-time adaptation in the discussion with intellild. The final change removed the disputed garbage-header handling. Do not encode the earlier centralized-parser proposal as accepted policy. For path handling, hardfist proposed targeted webpack alignment while intellild proposed internal URLs; that representation question remains a proposal. Read [tradeoffs](references/tradeoffs.md) when either boundary matters.

Reusing an existing helper is conditional. CPunisher also requested a smaller emitter to avoid unnecessary source-map generation. Public compatibility aliases have evidence, while stormslowly removed an obsolete internal protocol branch when both components shipped together. Match the boundary instead of choosing a universal reuse or compatibility rule.

## Write and review

Identify the observable contract and relevant owner evidence, then make the smallest supported change. Match validation to the failure: runtime execution, emitted output, rebuild/cache transitions, platform behavior, or performance measurement. Use the current repository commands and report what actually ran. Source-reported benchmark numbers in the ledgers were not independently reproduced.

In review, explain the trigger, consequence, and evidence for each substantive request. Separate correctness issues from optional design suggestions. Do not predict a maintainer's approval from this mode, an unexplained historical approval, or the activity ranking. Creating this mode does not authorize posting reviews, messaging contributors, or scheduling refreshes.
