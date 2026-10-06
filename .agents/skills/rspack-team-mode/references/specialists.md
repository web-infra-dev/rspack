# Specialized contributor evidence

Scope: public `web-infra-dev/rspack`, 2025-10-06 through 2026-10-06. Collection date 2026-10-06. Read the complete holdout registry before mining and excluded every listed PR. Bounded representative mining, not an exhaustive body crawl. Read 28 selected PR bodies, relevant actual patches, commit account attribution, and available review threads/replies. All selected PR files, review, comment and thread connections were exhausted. PR 11899 has 250 commits over three GraphQL pages, all collected. Inspected seven issue records and one related discussion. Outside-project links were context only and were not crawled. Approvals without rationale support activity only. Rules below describe observed practice, not the contributors' approval or universal team consensus.

## 9aoy

Active Rstest owner. Three authored training PRs; GitHub commit authors all link to 9aoy. No substantive authored issue discussion in this sample. Three cases support the scoped practice: exercise Rstest transformations at the runtime or build boundary that failed, including the unaffected path. Treat cache and runtime compatibility details below as concrete checks for similar work, not general preferences inferred from a single example.

- [15823](https://github.com/web-infra-dev/rspack/pull/15823), merged 2026-09-23, authored code and config cases. Routes mock APIs through the runtime registry, carries source origin for variable requests, keeps manual-mock export references alive, and adds generated-code checks for literal/imported/global APIs, variable requests, older-runtime rejection and tree-shaken exports. Optional runtime capabilities have separate flags. This is evidence of compatibility-aware transformation work, not a mandate to add flags to every feature.
- [15097](https://github.com/web-infra-dev/rspack/pull/15097), merged 2026-08-07, authored code and config case. Makes the `import.meta.rstest` resolver call optional so an isolated worker runtime without the resolver evaluates the guard to undefined. Tests both resolver presence and absence rather than assuming the parent test runtime represents every module.
- [13775](https://github.com/web-infra-dev/rspack/pull/13775), merged 2026-04-21, authored code and cache case. Replaces serialization-skipped spans with cacheable DependencyRange metadata and adds a NEXT_START regression. Bot review requested range terminology and an explanatory cache comment; final patch contains both. Do not attribute those review requests to 9aoy.

Candidate operational rule: for Rstest code generation changes, reproduce the failing runtime or restart state in the existing config/cache runners and check a nearby unaffected path. Cacheable source positions and missing runtime helpers deserve explicit checks when those boundaries change.

## Timeless0911

Active Rslib/build and library-output owner. Three authored training PRs, all commit accounts Timeless0911. One substantive reply to a human reviewer. The sample supports ownership and concrete practices, but no repeated personal hard rule across all three topics.

- [14575](https://github.com/web-infra-dev/rspack/pull/14575), merged 2026-06-25, authored configuration. Assigns separate runtime chunk names to index and worker multi-lib outputs through one helper. The code labels this a temporary upstream workaround. Counterexample to a universal prohibition on workarounds.
- [14215](https://github.com/web-infra-dev/rspack/pull/14215), merged 2026-06-02, authored options logic and ESM output regression. Infers modern-module CommonJS fallback from target nodeBuiltins capability rather than external presets so disabling a preset does not change the target's capabilities. A bot review flags mixed web/node null capability; final diff still uses truthiness. Do not claim comprehensive mixed-target correctness or universal null handling from this case.
- [15447](https://github.com/web-infra-dev/rspack/pull/15447), merged 2026-09-03, authored binding declaration and reply to chenjiahan. Changes the shared handwritten binding declaration to explicit .js imports instead of browser-only postprocessing. Timeless0911 reports checking byte-identical copied declarations and NodeNext/Bundler resolution. This is reported verification, not rerun here.

Tentative operational directions: when editing shared declarations, fix their source and check all consumers; for multi-lib output, check runtime filename collisions. These are useful case-specific checks, not independently established personal hard requirements.

## fi3ework

Active Rstest owner. Three authored training PRs, all linked commit accounts fi3ework; one authored issue. Repeated code and response evidence supports preserving mock evaluation semantics and testing concrete regressions. Two independent review conversations also show separating a reachable regression from a broader, preexisting limitation.

- [15877](https://github.com/web-infra-dev/rspack/pull/15877), merged 2026-09-28, authored parser and fixture plus replies. Adds non-hoisted mock APIs in expression positions. After feedback, preserves sibling declarator traversal and adds a multi-declarator sibling require case that the author reports failing before the fix. Explicitly leaves statement-level comma-sequence support out of scope because main already lacks it and widening the statement handler changes hoisted APIs.
- [14810](https://github.com/web-infra-dev/rspack/pull/14810), merged 2026-07-17, authored ordering change and transitive importActual fixtures. Registers mocks before evaluating the importActual subgraph while keeping imported actual bindings initialized before factory execution. Encodes the fragment order in comments and tests both transitive access and spreading actual exports into a mock.
- [14247](https://github.com/web-infra-dev/rspack/pull/14247), merged 2026-06-08, authored external-dynamic-import fix and emitted-code checks plus replies. Uses a clean request key to bridge distinct dynamic/static external IDs. Leaves internal imports unchanged. Rejects an inclusive-range bot finding by citing actual half-open replacement semantics and emitted code. Acknowledges a property-access external mechanism but demonstrates it is unreachable through Rstest's plain-string externalization, and documents that boundary instead of widening the patch.
- [15370](https://github.com/web-infra-dev/rspack/issues/15370), opened 2026-08-24, authored bug report. Gives Node/Rsbuild/Rspack versions and observable watch-rebuild symptoms after a plugin exception. Supports reproduction specificity, not a separate hard triage rule.

Operational rule: test Rstest transforms through evaluation order, transitive module behavior and sibling expression traversal, using a concrete failing fixture. Before broadening a fix in response to review, establish reachability in the actual Rstest pipeline and whether the behavior is a new regression. The latter has two explicit cases and remains a scoped review practice.

## yifancong

Active specialized Rsdoctor owner. Three authored PRs, all commits linked to yifancong; one issue with two dated replies. Two changes and human review replies support retaining structured compiler information until the reporting boundary. No blanket rule that every type needs its own unit test is supported.

- [15055](https://github.com/web-infra-dev/rspack/pull/15055), merged 2026-08-05, authored config regression. Asserts JSON sizes against final codeGenerationResults after tree shaking under source-map and cheap-module-source-map. Also asserts emitted source is smaller than original JSON-derived source.
- [13384](https://github.com/web-infra-dev/rspack/pull/13384), merged 2026-03-19, authored Rust/NAPI/TS changes and response to LingyuCoder. Keeps exports type as BuildMetaExportsType internally and converts with Display at the JS boundary. The title says dependency connections, while the actual change concerns exports type; use the patch for attribution.
- [12983](https://github.com/web-infra-dev/rspack/pull/12983), merged 2026-03-03, authored structured OptimizationBailoutItem and side-effect reporting changes plus config cases. When asked to avoid string allocation, explains why SideEffects cannot use as_str and instead accesses Message directly where appropriate. Boundary conversions retain existing string output for stats.
- [13596](https://github.com/web-infra-dev/rspack/issues/13596), replies 2026-04-03 and 2026-04-09. Reports inability to reproduce package installation, provides tested version/command, asks for environment or reproducible demo, then notes the user's retry succeeded. Closure actor not inferred.

Scoped direction supported across source kinds: preserve typed compiler facts while collecting Rsdoctor data, convert at the JS/reporting boundary, and validate reported values against compiler output. Reproduction request is a single observed triage case.

## SoonIter

Active specialized website/Rspress contributor. Three authored PRs, all commit accounts SoonIter, and one authored issue. One PR explicitly says it was generated by an automation, so it supports repository practice rather than unaided personal style.

- [15909](https://github.com/web-infra-dev/rspack/pull/15909), merged 2026-09-28, explicitly automation-generated ecosystem CI fix. Registers normalized options in builtinReferences for native inline loaders and adds lightningcss/CSS-extract coverage. The PR supplies a concrete failure signature and controlled before/after attribution. Do not treat the prose or investigation as a personal convention.
- [13770](https://github.com/web-infra-dev/rspack/pull/13770), merged 2026-04-21, authored shared BlogList and bilingual frontmatter changes. Replaces duplicated index entries with page metadata, language filtering, date order and fallback authors. Replies to a date report that zero-padding was fixed. Some surrounding metadata was changed, so do not infer a general refusal to edit data.
- [12498](https://github.com/web-infra-dev/rspack/pull/12498), merged 2025-12-18, authored website cleanup. Uses Rspress Link and default table components instead of custom navigation/table behavior; states the navigation and external-link rationale.
- [12080](https://github.com/web-infra-dev/rspack/issues/12080), opened 2025-11-04, authored loader-diagnostic report. Gives version, repository reproduction, commands and Rspack/webpack comparison. External reproduction was not crawled.

Tentative website direction: use Rspress navigation/table behavior and derive repeated content from shared page metadata. Two direct website cases support this; do not promote it into an all-team hard rule.

## elecmonkey

Active specialized library-output/worker contributor. Three authored PRs with all commits linked to elecmonkey; one authored issue. Three independent changes support mode-specific fixes and emitted-output regressions.

- [15670](https://github.com/web-infra-dev/rspack/pull/15670), merged 2026-09-15, authored external URL dependency template and config case. Reuses the worker-external approach to render the external request and remove the connection from the graph only where new-url-relative semantics permit it. Other modes retain the normal dependency template.
- [15155](https://github.com/web-infra-dev/rspack/pull/15155), merged 2026-08-12, authored hoisting condition and ESM snapshots. Allows a NewUrl edge specifically under NewUrlRelative; tests one asset referenced by both ESM import and new URL to catch unwanted require/runtime output.
- [15012](https://github.com/web-infra-dev/rspack/pull/15012), merged 2026-07-31, authored SharedWorker option classification and tests plus self-review replies. Distinguishes strings, objects and unknown expressions, preserves single evaluation, and routes spread arguments through the runtime wrapper. Regression covers classic and module output. A self-review comment is evidence of explicit reasoning, not independent reviewer agreement.
- [15270](https://github.com/web-infra-dev/rspack/issues/15270), opened 2026-08-21, authored report with parser options, commands, input/output comparison and a reproduction link. Documents createRequire folding import.meta.url despite disabled importMeta parsing.

Operational direction: limit library/worker transforms to the exact parser and output modes that need them; verify emitted ESM and nearby syntax alternatives, including mixed references and spread/unknown argument forms.

## ScriptedAlchemy

Active specialized Module Federation owner. Three authored PRs, one issue and its related discussion. The two startup PRs have commits linked to both ScriptedAlchemy and claude; treat implementation choices as coauthored work. PR 13157 commits link to ScriptedAlchemy. The issue and discussion are one design case, not independent confirmations.

- [13157](https://github.com/web-infra-dev/rspack/pull/13157), merged 2026-03-03, authored integration branch changes and serial HTTP cases. Serial host tests compare remote moduleLoading, JS and CSS metadata and scoped client references. The PR also contains unrelated cleanup. Avoid assuming all paths implement the title's narrow change.
- [11899](https://github.com/web-infra-dev/rspack/pull/11899), merged 2026-01-21, coauthored async startup implementation. Keeps container get/init exports synchronous while gating non-container entries. In reply to ahabhgk, accepts general ASYNC_STARTUP and direct chunk-loader plugin changes rather than keeping an MF-specific shortcut. The final diff shows ASYNC_STARTUP controlling async chunk-loading behavior.
- [12154](https://github.com/web-infra-dev/rspack/pull/12154), merged 2025-11-13, coauthored duplicate-startup-wrapper prevention. Adds a runtime capability flag and startup requirement handling. This older MF-specific design is superseded by the more general approach above; do not preserve its exact flag as a style rule.
- [15573](https://github.com/web-infra-dev/rspack/issues/15573), opened 2026-09-09, and [discussion 15576](https://github.com/web-infra-dev/rspack/discussions/15576), same RFC. Explains layer identity through resolution, runtime initialization and manifest metadata; separates compile-time layer selection from runtime scope/realm negotiation and preserves unlayered behavior. Proposal evidence is not proof all proposed behavior shipped.

Scoped direction: for federation startup or identity changes, trace compiler/runtime/manifest effects and exercise host/remote integration while preserving existing container contracts. Attribution is to observed coauthored integration practice and explicit design rationale, not exclusive personal code authorship or universal team preference.

## GiveMe-A-Name

Sparse current authorship, with watcher review participation. One authored training PR, two other-author reviewed PRs, one authored issue. No hard personal rule from this sample.

- [13491](https://github.com/web-infra-dev/rspack/pull/13491), merged 2026-04-10, all commits linked to GiveMe-A-Name. Hashes chunk content hashes into compilation fullhash, sorts map-backed entries by SourceType and hashes both type and digest. Explains to CPunisher that hashing the type does not make iteration deterministic, then uses derived ordering and a key sort instead of imposing ordering on the digest. Adds a two-build CSS-only hash case. Replies that unrelated edits were removed, but a trailing newline .gitignore diff remains; do not claim perfect scope cleanliness.
- [12185](https://github.com/web-infra-dev/rspack/pull/12185), merged 2025-11-13, and [11948](https://github.com/web-infra-dev/rspack/pull/11948), merged 2025-10-22. Authorship and commits belong to h-a-n-a. GiveMe-A-Name's reviews have empty bodies. These support participation, not a testing or architecture preference.
- [13304](https://github.com/web-infra-dev/rspack/issues/13304), opened 2026-03-11, authored ESM SSR cache-invalidation report with reproduction steps and expected/actual behavior.

Tentative case-specific direction: when hashing map-backed data, order entries by their semantic key before hashing key and value, and verify CSS-only changes across builds. One substantive PR is insufficient for a standing personal hard rule.

## swwind

Occasional specialized contributor, not established as an active core owner by this sample. Three authored PRs, all commits linked to swwind; one in-window issue reply. No hard personal rule.

- [14287](https://github.com/web-infra-dev/rspack/pull/14287), merged 2026-06-24, adds VarDeclKind::Var checks for for-in/of hoist collection and extends the existing dead-code elimination case. A narrow language-semantics fix.
- [14447](https://github.com/web-infra-dev/rspack/pull/14447), merged 2026-06-24, wires GNU/musl RISC-V bindings through build/release/package configuration and bilingual supported-platform docs.
- [14442](https://github.com/web-infra-dev/rspack/pull/14442), merged 2026-06-16, documents manually installing the Wasm fallback after testing without the native package. Revises "any platform" to "most platforms" after hardfist's feedback.
- [11656](https://github.com/web-infra-dev/rspack/issues/11656#issuecomment-4717213918), comment 2026-06-16, identifies the Wasm fallback and missing documentation. The issue body predates the window and is context only.

Tentative direction: platform support claims should distinguish verified native packages from fallback behavior and update packaging plus docs together. The parser change does not independently support this platform pattern.

## inottn

Sparse contribution; active core ownership unestablished. One authored PR with commits linked to inottn. No substantive dated issue activity in discovery and no hard rules.

- [12570](https://github.com/web-infra-dev/rspack/pull/12570), merged 2025-12-28, fixes CSS emission for universal targets. Distinguishes document === false from falsy merged capability values, centralizes CSS defaults, extends existing CSS config cases, and adjusts the runner to recognize arrays containing web/webworker. Links an exact webpack implementation revision for context; external code not crawled.

Tentative direction for similar changes: preserve the difference between false and mixed-target capability values and make sure the existing test runner actually executes the intended target. One case does not establish a personal review policy.

## Validation limits

Training excludes all 12 reserved PRs, including Timeless0911 13861 and fi3ework 15689. This worker did not perform holdout evaluation. Parent owns it. No heldout case was invented for accounts without one. Code-writing observations are patch-backed but no historical tests were rerun. Several rules are intentionally tentative or absent.
