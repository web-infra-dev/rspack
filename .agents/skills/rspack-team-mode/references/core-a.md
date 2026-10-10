# Core-A maintainer mining

Scope: PUBLIC `web-infra-dev/rspack`, 2025-10-06 through 2026-10-06. No external repository crawls. Entire parent holdout PR list excluded before sampling. Retrieval uses authenticated gh. This is evidence of observed account activity, not private belief or permission to represent maintainers.

Sampling: 22 substantive PRs, with paginated files/commits/reviews/inline comments and issue-style PR replies for 21 (PR 15366 lacks issue-style reply fetch). Read focused changed implementation hunks, regression hunks, and relevant full human threads. Large PR patches were not exhaustively audited. Six account identities verified with users endpoint (all type User). Source status below distinguishes merged from proposals. No approval-only rule evidence. Bots excluded as rule authors; their comments remain context for human responses.

## Coverage
- hardfist: GitHub id 8898718, 228 merged PR metadata candidates, 92 non-holdout inline comments indexed; 4 deep training PR cases. Profile https://github.com/hardfist.
- chenjiahan: GitHub id 7237365, 502 merged PR metadata candidates, 223 non-holdout inline comments indexed; 5 deep training PR cases. Profile https://github.com/chenjiahan.
- LingyuCoder: GitHub id 2663351, 405 merged PR metadata candidates, 320 non-holdout inline comments indexed; 4 deep training PR cases. Profile https://github.com/LingyuCoder.
- stormslowly: GitHub id 415655, 187 merged PR metadata candidates, 51 non-holdout inline comments indexed; 4 deep training PR cases. Profile https://github.com/stormslowly.
- intellild: GitHub id 8379858, 108 merged PR metadata candidates, 52 non-holdout inline comments indexed; 4 deep training PR cases. Profile https://github.com/intellild.
- SyMind: GitHub id 19852293, 135 merged PR metadata candidates, 28 non-holdout inline comments indexed; 4 deep training PR cases. Profile https://github.com/SyMind.

## hardfist

Observed ownership: Webpack compatibility, cache/snapshots, core lifecycle review.

Candidate operational convention: For compatibility work, inspect actual upstream behavior and the exact lifecycle/option semantics before changing behavior; isolate independently motivated hardening. Supported across #13197, #15049, #15271 plus lifecycle mismatch question #14757. The public upstream links are rationale in these Rspack discussions; their external repositories were not crawled.

### #13197 , feat: Implement HashedModuleIdsPlugin
- Source: https://github.com/web-infra-dev/rspack/pull/13197; date 2026-03-31T02:35:57Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-03-31T16:33:32Z.
- Attribution: PR opened by aancuta; commit linked-account counts {'unlinked': 10, 'aancuta': 2}. Merge commits/collaborator code not treated as sole authorship. Files fetched 39.
- Observation: Review corrects stale webpack docs: hashed module IDs are supported by the actual upstream types. Final patch adds hashed option, plugin, defaults/options tests.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/13197#discussion_r3013122199, https://github.com/web-infra-dev/rspack/pull/13197#discussion_r3014708506.

### #15049 , feat: align persistent cache storage options with webpack
- Source: https://github.com/web-infra-dev/rspack/pull/15049; date 2026-08-03T07:56:58Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-08-04T06:30:24Z.
- Attribution: PR opened by hardfist; commit linked-account counts {'hardfist': 6}. Merge commits/collaborator code not treated as sole authorship. Files fetched 21.
- Observation: Authored cache storage options use path.resolve(directory,name); reply refuses path.join/containment tightening because it changes compatibility, and final patch preserves compiler-index disambiguation. Docs clarify storage.location.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/15049#discussion_r3702204644, https://github.com/web-infra-dev/rspack/pull/15049#discussion_r3703473318, https://github.com/web-infra-dev/rspack/pull/15049#discussion_r3703474711.

### #15271 , feat(cache): add module snapshots to new cache
- Source: https://github.com/web-infra-dev/rspack/pull/15271; date 2026-08-24T04:26:16Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; NOT MERGED.
- Attribution: PR opened by hardfist; commit linked-account counts {'hardfist': 12}. Merge commits/collaborator code not treated as sole authorship. Files fetched 16.
- Observation: Unmerged snapshot proposal retains context child names even for immutable content and propagates non-NotFound errors. This is stated approach, not proof of landed implementation.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/15271#discussion_r3840696634, https://github.com/web-infra-dev/rspack/pull/15271#discussion_r3840697047.

### #14757 , fix(watcher): serialize watch and close on the native watcher
- Source: https://github.com/web-infra-dev/rspack/pull/14757; date 2026-07-14T06:15:36Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; NOT MERGED.
- Attribution: PR opened by stormslowly; commit linked-account counts {'stormslowly': 3}. Merge commits/collaborator code not treated as sole authorship. Files fetched 7.
- Observation: Review asks whether pause still panics/aborts and notes native API semantics differ from JS. Do not treat a narrow watch/close fix as complete lifecycle compatibility.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14757#discussion_r3576474630, https://github.com/web-infra-dev/rspack/pull/14757#discussion_r3576495521, https://github.com/web-infra-dev/rspack/pull/14757#discussion_r3576606010.


## chenjiahan

Observed ownership: Public TypeScript API, compatibility/deprecation, bilingual user documentation.

Candidate operational convention: Review public changes as user contracts: keep compatible aliases during renames/stabilization, specify exact defaults/version boundaries, and keep English/Chinese migration guidance aligned. #12483, #12532, #15049 support this. Extensible named option/context objects appear in #12277 and #12672; this narrower shape preference has only two independent cases and should remain a conditional suggestion, not a universal blocker.

### #12277 , feat(loader-runner): Allow limiting worker pool size for parallel loaders
- Source: https://github.com/web-infra-dev/rspack/pull/12277; date 2025-11-26T05:57:05Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2025-11-28T06:21:14Z.
- Attribution: PR opened by Pablinho; commit linked-account counts {'Pablinho': 2}. Merge commits/collaborator code not treated as sole authorship. Files fetched 9.
- Observation: Recommends boolean | {maxWorkers?} and generic worker terminology instead of exposing tinypool thread details; final patch adopts it.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/12277#discussion_r2563345952, https://github.com/web-infra-dev/rspack/pull/12277#discussion_r2563346950.

### #12483 , feat: stabilize SubresourceIntegrityPlugin
- Source: https://github.com/web-infra-dev/rspack/pull/12483; date 2025-12-17T05:46:42Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2025-12-17T06:32:47Z.
- Attribution: PR opened by LingyuCoder; commit linked-account counts {'LingyuCoder': 6}. Merge commits/collaborator code not treated as sole authorship. Files fetched 15.
- Observation: Requests retaining experiments.SubresourceIntegrityPlugin as deprecated compatibility alias; final exports retain old route and add stable export.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/12483#discussion_r2625684350.

### #12532 , refactor: extract case-sensitive check to plugin
- Source: https://github.com/web-infra-dev/rspack/pull/12532; date 2025-12-23T08:35:41Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2025-12-23T08:58:46Z.
- Attribution: PR opened by LingyuCoder; commit linked-account counts {'LingyuCoder': 6, 'chenjiahan': 2}. Merge commits/collaborator code not treated as sole authorship. Files fetched 23.
- Observation: Supplies paired English/Chinese rename/deprecation/version wording and accurate warning behavior; final patch includes alias and matched docs.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/12532#discussion_r2642392808, https://github.com/web-infra-dev/rspack/pull/12532#discussion_r2642395064, https://github.com/web-infra-dev/rspack/pull/12532#discussion_r2642395513.

### #12672 , feat(css): introduce `resolveImport` parser option for css parser to choose whether to resolve `@import` syntax
- Source: https://github.com/web-infra-dev/rspack/pull/12672; date 2026-01-09T05:08:15Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-02-09T07:55:25Z.
- Attribution: PR opened by JSerFeng; commit linked-account counts {'JSerFeng': 1}. Merge commits/collaborator code not treated as sole authorship. Files fetched 21.
- Observation: Suggests context object over flat CSS import callback parameters and asks default=true; patch carries the named context and documents default.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/12672#discussion_r2674839187, https://github.com/web-infra-dev/rspack/pull/12672#discussion_r2674840749.

### #15049 , feat: align persistent cache storage options with webpack
- Source: https://github.com/web-infra-dev/rspack/pull/15049; date 2026-08-03T08:58:41Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-08-04T06:30:24Z.
- Attribution: PR opened by hardfist; commit linked-account counts {'hardfist': 6}. Merge commits/collaborator code not treated as sole authorship. Files fetched 21.
- Observation: Flags unintuitive cache-location explanation and missing version metadata, resulting in author docs revisions.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/15049#discussion_r3702568984, https://github.com/web-infra-dev/rspack/pull/15049#discussion_r3702570212.


## LingyuCoder

Observed ownership: Compilation performance, benchmarks, incremental graph behavior, identifiers/runtime.

Candidate operational convention: Keep performance work semantically bounded: benchmark the intended operation with minimal setup outside timing; avoid coupling benchmarks to private compiler state; preserve established identifier behavior while reducing allocations. #13594/#13702 authored benchmark code and #13788 authored refactor plus issue #14768 substantiate the approach. Do not encode the discarded separate-probe design as final guidance.

### #13594 , test(benchmark): add persistent cache codspeed cases
- Source: https://github.com/web-infra-dev/rspack/pull/13594; date 2026-04-02T12:39:35Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-04-03T07:45:48Z.
- Attribution: PR opened by LingyuCoder; commit linked-account counts {'LingyuCoder': 11}. Merge commits/collaborator code not treated as sole authorship. Files fetched 3.
- Observation: Initial replies argued separate correctness probes to avoid warming measured cache; final reply removes all private restore-state assertions after jerrykingxyz review. Landed benchmark seeds then times restore, defers cleanup outside timing.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/13594#discussion_r3027803910, https://github.com/web-infra-dev/rspack/pull/13594#discussion_r3027804044, https://github.com/web-infra-dev/rspack/pull/13594#discussion_r3027804240.

### #13629 , refactor(javascript): parallelize module concatenation search
- Source: https://github.com/web-infra-dev/rspack/pull/13629; date 2026-04-07T08:14:10Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; NOT MERGED.
- Attribution: PR opened by LingyuCoder; commit linked-account counts {'LingyuCoder': 3}. Merge commits/collaborator code not treated as sole authorship. Files fetched 0.
- Observation: Unmerged/reverted parallel concatenation search was revised to ordered Rayon-sized batches, local cache retained absent profiling need; final PR has no diff. Counterexample to portraying parallelization as accepted strategy.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/13629#discussion_r3043698664, https://github.com/web-infra-dev/rspack/pull/13629#discussion_r3043698935, https://github.com/web-infra-dev/rspack/pull/13629#discussion_r3043699199.

### #13702 , test(benchmark): add compilation stage benchmark cases
- Source: https://github.com/web-infra-dev/rspack/pull/13702; date 2026-04-15T08:22:01Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-04-16T07:25:28Z.
- Attribution: PR opened by LingyuCoder; commit linked-account counts {'LingyuCoder': 11}. Merge commits/collaborator code not treated as sole authorship. Files fetched 6.
- Observation: Uses benchmark-local NoopCache; gates public pass reexports behind benchmark-passes; restores assets via delete_asset/emit_asset and sorts seed identifiers before truncation. Final code adopts feature gate.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/13702#discussion_r3085006136, https://github.com/web-infra-dev/rspack/pull/13702#discussion_r3085006448, https://github.com/web-infra-dev/rspack/pull/13702#discussion_r3085006719.

### #13788 , perf: reduce string churn across identifiers, ids, and runtime helpers
- Source: https://github.com/web-infra-dev/rspack/pull/13788; date 2026-04-22T09:44:44Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-04-23T05:06:46Z.
- Attribution: PR opened by LingyuCoder; commit linked-account counts {'LingyuCoder': 12}. Merge commits/collaborator code not treated as sole authorship. Files fetched 13.
- Observation: Refactor writes into reused buffers/Cow and deliberately preserves regexp-like slash-wrapped segments. Documents append semantics instead of changing existing predicate behavior. Existing historical inline Rust tests are not permission to override current repository test policy.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/13788#discussion_r3123033711, https://github.com/web-infra-dev/rspack/pull/13788#discussion_r3123033715, https://github.com/web-infra-dev/rspack/pull/13788#discussion_r3123033724.


## stormslowly

Observed ownership: Native watcher/lazy compilation lifecycle, performance and path infrastructure.

Candidate operational convention: Check proposed performance and lifecycle changes against focused measurements and reproductions; accept or reject individual micro-optimizations using observed data, and re-test revised synchronization across repeated calls/close. #14705/#14757/#15205 give three independent cases. The bounded-retention race accepted in #15205 is a case-specific tradeoff, never permission for unsafe races.

### #12678 , refactor(lazy-compilation): use POST request to transfer  ids of active modules
- Source: https://github.com/web-infra-dev/rspack/pull/12678; date 2026-01-14T04:11:04Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-01-15T05:42:08Z.
- Attribution: PR opened by stormslowly; commit linked-account counts {'stormslowly': 28}. Merge commits/collaborator code not treated as sole authorship. Files fetched 14.
- Observation: Initially wanted old/new lazy middleware compatibility, then removed non-POST branch because runtime and middleware ship together. This limits blanket compatibility prescriptions.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/12678#discussion_r2688871722, https://github.com/web-infra-dev/rspack/pull/12678#discussion_r2689120013.

### #14705 , perf(mangle_exports): pre-size per-module maps and vecs
- Source: https://github.com/web-infra-dev/rspack/pull/14705; date 2026-07-06T14:33:06Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-07-07T03:22:56Z.
- Attribution: PR opened by stormslowly; commit linked-account counts {'stormslowly': 2}. Merge commits/collaborator code not treated as sole authorship. Files fetched 1.
- Observation: Leaves nested_exports unreserved after measured instruction count worsens with with_capacity, while pre-sizing other collections. Exact landed patch corroborates selective optimization.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14705#discussion_r3529677153.

### #14757 , fix(watcher): serialize watch and close on the native watcher
- Source: https://github.com/web-infra-dev/rspack/pull/14757; date 2026-07-13T03:39:08Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; NOT MERGED.
- Attribution: PR opened by stormslowly; commit linked-account counts {'stormslowly': 3}. Merge commits/collaborator code not treated as sole authorship. Files fetched 7.
- Observation: Reports initial abort+await fix then falsifies it with repeated-watch regression (detached predecessor chain); revised mutex passes reported runs. PR not merged and superseded by #14829, so use diagnostic/testing rationale only.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14757#discussion_r3567903310, https://github.com/web-infra-dev/rspack/pull/14757#discussion_r3568899640.

### #15205 , perf(paths): intern ArcPath as a single-allocation prehashed path
- Source: https://github.com/web-infra-dev/rspack/pull/15205; date 2026-08-19T04:49:26Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-08-19T08:27:00Z.
- Attribution: PR opened by stormslowly; commit linked-account counts {'stormslowly': 12}. Merge commits/collaborator code not treated as sole authorship. Files fetched 34.
- Observation: Author keeps a bounded interner retention race rather than shard write lock on each destructor, citing -3.14% peak RSS/-0.45% cycles; defends Archive and rt-multi-thread feature requirements via build failures. Landed path interning patch.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/15205#discussion_r3810125945, https://github.com/web-infra-dev/rspack/pull/15205#discussion_r3811545995, https://github.com/web-infra-dev/rspack/pull/15205#discussion_r3811556564.


## intellild

Observed ownership: CSS/CopyPlugin, JavaScript environment semantics, source-map integration and loader cache.

Candidate operational convention: Keep compatibility fixes scoped to the actual producer/consumer contract and add focused regressions for edge semantics: glob escaping/case handling, env own-properties/namespace boundaries, fresh-run versus watch invalidation. #14023/#14483/#15366 patches substantiate this; CSS tracking issue #14002 adds explicit scope splits and regression expectations. Do not adopt every account reply as personal prose: several identify OpenAI Codex assistance.

### #14023 , fix(copy-plugin): support JS input file system for glob copies
- Source: https://github.com/web-infra-dev/rspack/pull/14023; date 2026-05-13T03:39:36Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-05-18T14:21:30Z.
- Attribution: PR opened by intellild; commit linked-account counts {'intellild': 14}. Merge commits/collaborator code not treated as sole authorship. Files fetched 18.
- Observation: Distinguishes filesystem Windows separators from pattern escapes; final patch normalizes source paths then escapes glob metacharacters; regression assertions cover literal * and ? and dot-case sensitivity.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14023#discussion_r3231332433, https://github.com/web-infra-dev/rspack/pull/14023#discussion_r3231332689, https://github.com/web-infra-dev/rspack/pull/14023#discussion_r3259461026.

### #14483 , feat: support import.meta.env
- Source: https://github.com/web-infra-dev/rspack/pull/14483; date 2026-06-29T11:16:03Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-07-22T02:47:27Z.
- Attribution: PR opened by intellild; commit linked-account counts {'intellild': 14}. Merge commits/collaborator code not treated as sole authorship. Files fetched 59.
- Observation: Limits env accumulation to declared sources, does not mirror arbitrary process.env definitions, emits __proto__ as own property, normalizes edge cases on JS side. Dedicated config cases implement regressions. SWC evaluation pipeline explicitly deferred.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14483#discussion_r3491259873, https://github.com/web-infra-dev/rspack/pull/14483#discussion_r3491260522, https://github.com/web-infra-dev/rspack/pull/14483#discussion_r3496414047.

### #14621 , perf: build rspack source maps directly
- Source: https://github.com/web-infra-dev/rspack/pull/14621; date 2026-07-02T03:30:48Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-07-02T07:33:37Z.
- Attribution: PR opened by intellild; commit linked-account counts {'intellild': 14}. Merge commits/collaborator code not treated as sole authorship. Files fetched 6.
- Observation: Initially defends centralized garbage-header parsing for compatibility in a Codex-labelled reply; after SyMind pushes spec and bundler scope, author removes support. Treat as resolved disagreement, not standing centralized-parser rule.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14621#discussion_r3510285251, https://github.com/web-infra-dev/rspack/pull/14621#discussion_r3510285586, https://github.com/web-infra-dev/rspack/pull/14621#discussion_r3510947692.

### #15366 , refactor(loader): reuse filesystem snapshots for cache dependencies
- Source: https://github.com/web-infra-dev/rspack/pull/15366; date 2026-08-28T04:53:13Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-08-28T05:25:56Z.
- Attribution: PR opened by intellild; commit linked-account counts {'intellild': 9}. Merge commits/collaborator code not treated as sole authorship. Files fetched 24.
- Observation: Fresh build invalidates all memoized filesystem metadata; rebuild invalidates modified/removed paths. Cache entries retain their own validation; authored commits and cache/watch regression changes corroborate reply.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/15366#discussion_r3878006124.


## SyMind

Observed ownership: rspack_sources/source-map architecture, diagnostic source lifetimes; RSC package responsibility from issue record.

Candidate operational convention: Keep expensive derived work and adaptation at the responsible boundary: cheap OriginalSource construction, CachedSource wrapping at Rspack level rather than computation cache in SourceMapSource, parse versus fetch distinction, and render-scoped source adapters owning flattened Cow. Four independent reviews substantiate the boundary principle; three of these PRs unmerged, so these are review expectations, not claims that all proposals landed.

### #14436 , perf(sources): speed up source map tokenization
- Source: https://github.com/web-infra-dev/rspack/pull/14436; date 2026-06-16T08:15:31Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; NOT MERGED.
- Attribution: PR opened by nnnnoel; commit linked-account counts {'nnnnoel': 2}. Merge commits/collaborator code not treated as sole authorship. Files fetched 2.
- Observation: Review says no computation in OriginalSource construction; final proposal moves ASCII check to chunk streaming. Unmerged.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14436#discussion_r3419133661.

### #14445 , perf(sources): cache generated source info
- Source: https://github.com/web-infra-dev/rspack/pull/14445; date 2026-06-17T02:34:54Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; NOT MERGED.
- Attribution: PR opened by nnnnoel; commit linked-account counts {'nnnnoel': 3}. Merge commits/collaborator code not treated as sole authorship. Files fetched 3.
- Observation: Review says SourceMapSource should not hold computed cache state; use CachedSource at Rspack layer. Author measures intended wrapper case and agrees internal cache provides no meaningful benefit. Unmerged proposal wraps minimizer outputs.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14445#discussion_r3425209339.

### #14621 , perf: build rspack source maps directly
- Source: https://github.com/web-infra-dev/rspack/pull/14621; date 2026-07-02T02:59:42Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; merged 2026-07-02T07:33:37Z.
- Attribution: PR opened by intellild; commit linked-account counts {'intellild': 14}. Merge commits/collaborator code not treated as sole authorship. Files fetched 6.
- Observation: Distinguishes ECMA-426 FetchSourceMap garbage stripping from ParseSourceMap, rejects unused helper and duplicate ignoreList control. Author ultimately removes header support in merged PR.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/14621#discussion_r3510177567, https://github.com/web-infra-dev/rspack/pull/14621#discussion_r3510248129, https://github.com/web-infra-dev/rspack/pull/14621#discussion_r3510288860.

### #15069 , fix: avoid retaining flattened diagnostic sources
- Source: https://github.com/web-infra-dev/rspack/pull/15069; date 2026-08-05T08:40:03Z; PUBLIC; kinds: changed code, commits, inline reviews/replies; NOT MERGED.
- Attribution: PR opened by hardfist; commit linked-account counts {'hardfist': 1}. Merge commits/collaborator code not treated as sole authorship. Files fetched 28.
- Observation: Suggests external render-scoped SourceCode/Diagnostic adapter preserving Cow lifetime, avoiding pervasive GraphicalReportHandler special cases and duplicated cause/footer logic. Unmerged; final proposal inspected but recommendation not claimed accepted.
- Direct account evidence: https://github.com/web-infra-dev/rspack/pull/15069#discussion_r3719161606.

## Issue sampling and limits

For each account queried is:issue commenter:<login> updated:2025-10-06..2026-10-06 sorted updated, top 5 only. Search totals are discovery, not an exhaustive comment audit; earlier comment dates filtered when reading. Fetched all comment pages on issues 14723,15950,14768,14446,14002,15144. No label/closure actions attributed without actor event verification.
- hardfist: search total 47, first 5 metadata results.
- chenjiahan: search total 77, first 5 metadata results.
- LingyuCoder: search total 72, first 5 metadata results.
- stormslowly: search total 32, first 5 metadata results.
- intellild: search total 18, first 5 metadata results.
- SyMind: search total 20, first 5 metadata results.
- SyMind: https://github.com/web-infra-dev/rspack/issues/15144#issuecomment-5248418867, 2026-08-11T02:46:22Z, PUBLIC issue comment. SyMind confirms no ./plugin entry needed and completion; LingyuCoder distinguishes published user availability from pending upstream maintenance merge.
- LingyuCoder: https://github.com/web-infra-dev/rspack/issues/15144#issuecomment-5582271731, 2026-09-08T09:03:10Z, PUBLIC issue comment. SyMind confirms no ./plugin entry needed and completion; LingyuCoder distinguishes published user availability from pending upstream maintenance merge.
- hardfist: https://github.com/web-infra-dev/rspack/issues/14723#issuecomment-4926998901, 2026-07-09T15:50:09Z, PUBLIC issue comment. AI-generated triage brief under hardfist account: persistent cache must recover writes or surface failure. Not used as independent personal-rule evidence.
- chenjiahan: https://github.com/web-infra-dev/rspack/issues/15950#issuecomment-6007663779, 2026-10-06T01:49:07Z, PUBLIC issue comment. Thanks-only reply; no convention evidence.
- stormslowly: https://github.com/web-infra-dev/rspack/issues/14446#issuecomment-4769300442, 2026-06-22T14:21:48Z, PUBLIC issue comment. stormslowly identifies Windows POSIX-style context root cause and precise workaround; diagnostic evidence only.
- intellild: https://github.com/web-infra-dev/rspack/issues/14002#issuecomment-4498019832, 2026-05-20T11:44:02Z, PUBLIC issue comment. intellild tracks CSS work, splits reverted experiments into smaller followups and records minimal reproductions plus expected inherited-condition semantics.
- intellild: https://github.com/web-infra-dev/rspack/issues/14002#issuecomment-4714585489, 2026-06-16T03:27:36Z, PUBLIC issue comment. intellild tracks CSS work, splits reverted experiments into smaller followups and records minimal reproductions plus expected inherited-condition semantics.
- intellild: https://github.com/web-infra-dev/rspack/issues/14002#issuecomment-5578503851, 2026-09-08T03:08:53Z, PUBLIC issue comment. intellild tracks CSS work, splits reverted experiments into smaller followups and records minimal reproductions plus expected inherited-condition semantics.
- LingyuCoder: https://github.com/web-infra-dev/rspack/issues/14768#issuecomment-5503479468, 2026-09-02T02:35:38Z, PUBLIC issue comment. LingyuCoder explains topology-aware chunk reuse, regression and release version; corroborates performance correctness.

Omissions: issues sampled, not exhaustively paginated search; discussions handled by parent and not used here; authored record is representative not exhaustive; no independent review or code-generation evaluation in this mining worker; no builds/tests executed. Reported performance/test numbers belong to source authors and were not reproduced. Historical PR code (including Rust test placement) does not supersede current repository AGENTS.md. All hard rules should remain bounded to supported areas; no assertion of six-person unanimity.

## Synthesis cautions

- Preserve resolved counterexamples: #13594 dropped private-state probes; #13629 parallelization reverted; #14757 superseded; #14621 garbage-header handling removed after disagreement.
- Compatibility differs by boundary: public alias retention has evidence, but stormslowly removed obsolete internal mixed-version branch when components ship together.
- Benchmarks and tests validate different claims: final #13594 explicitly leaves functional correctness to dedicated tests.
- Source-map layering is specialized SyMind evidence; do not attribute that whole architecture to every sampled contributor.
- Account-linked commits establish public attribution, not proof every line was manually written. Intellild explicitly discloses Codex assistance in several replies; bot text alone is never a maintainer preference.
