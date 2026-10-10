# Core-B evidence mining
Scope: public `web-infra-dev/rspack`, 2025-10-06 through 2026-10-06. Cutoff 2026-10-06. All 12 parent holdout PR conversations excluded before mining. No external repository content crawled. This is observed practice, not statements on behalf of maintainers.
Coverage: 9 verified human GitHub profiles; 33 PR cases; 207 authenticated REST responses, with pagination for files/commits/reviews/inline/issue-comments. Metadata census uses parent complete 2734 merged PRs. Examined relevant actual patches and replies; not every line of every diff. No independent issue crawl; later read three parent-selected discussion topics, no code execution, no independent evaluation here. No claim of exhaustive behavior capture. Raw responses were retained in the local run record; the public URLs are the durable evidence sources.

## ahabhgk
Profile: https://github.com/ahabhgk; API confirms login `ahabhgk`, type `User`. Census: 133 merged PRs including reserved cases, 71 nonheld-out inline comments.
Ownership/evidence limitation: Four authored cases; cache/hash invalidation appears across three. Strict-this behavior is one independent change and cannot establish a broad personal API policy. #14877 reports tests not run, so do not infer universal author test execution.
Supported operational convention: for incremental/cache changes, trace every input that can change generated results (dependency mutations, runtime identity, emitted path) into invalidation or reuse checks, and use a rebuild case when the failure is stateful (#12009/#13798/#14877).

- Source: https://github.com/web-infra-dev/rspack/pull/12009; date 2025-10-27 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `ahabhgk`; linked commit authors: ahabhgk.
  Observation: Adds DependencyUpdate mutations from affected dependencies before export analysis; removes the regression fixture workaround disabling providedExports.

- Source: https://github.com/web-infra-dev/rspack/pull/13328; date 2026-03-12 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `ahabhgk`; linked commit authors: ahabhgk.
  Observation: Propagates strictThisContextOnImports through dynamic imports, CommonJS require, and context dependencies, with assertions for this binding and used exports under both option values.

- Source: https://github.com/web-infra-dev/rspack/pull/14877; date 2026-07-20 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `ahabhgk`; linked commit authors: ahabhgk.
  Observation: Includes output path in render-cache reuse checks for JavaScript, CSS and extracted CSS; watch regression moves a chunk directory and checks relative asset URL changes.

- Source: https://github.com/web-infra-dev/rspack/pull/13798; date 2026-04-22 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `ahabhgk`; linked commit authors: ahabhgk.
  Observation: Includes RuntimeKey in module graph hash cache keys and revises exports-info hashing for stable hashing.

## JSerFeng
Profile: https://github.com/JSerFeng; API confirms login `JSerFeng`, type `User`. Census: 192 merged PRs including reserved cases, 173 nonheld-out inline comments.
Ownership/evidence limitation: Four authored cases and substantive review replies. Strong evidence for ESM semantic preservation and matching regression observables; snapshots and runtime tests are alternatives chosen by behavior, not universal mandates.
Supported operational convention: preserve ESM linking/evaluation semantics across externals, CSS, lazy contexts and runtime variants; match evidence to the failure,execute timing-sensitive behavior and check emitted content/paths for layout regressions (#13424/#13670/#14144/#15846). Preserve existing runtime handler composition when adding ESM paths.

- Source: https://github.com/web-infra-dev/rspack/pull/13424; date 2026-03-20 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `JSerFeng`; linked commit authors: JSerFeng.
  Observation: Expands ESM linking regression fixtures for duplicated external imports, namespace/default/named conflicts and preserveModules; changes linker classification and diagnoses invalid multiple-entry preservation.

- Source: https://github.com/web-infra-dev/rspack/pull/13670; date 2026-04-09 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `JSerFeng`; linked commit authors: JSerFeng.
  Observation: Handles CSS and assets by source type in preserveModules; author replies tighten CSS-only classification rather than excluding every non-JavaScript module and strengthen emitted CSS path/content snapshots.
  Interaction: https://github.com/web-infra-dev/rspack/pull/13670#discussion_r3092900441 (2026-04-16T11:40:46Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13670#discussion_r3092901178 (2026-04-16T11:40:56Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13670#discussion_r3092901811 (2026-04-16T11:41:04Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13670#discussion_r3092902844 (2026-04-16T11:41:16Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13670#discussion_r3092903449 (2026-04-16T11:41:23Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13670#discussion_r3092904888 (2026-04-16T11:41:38Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13670#discussion_r3097616376 (2026-04-17T03:01:19Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/14144; date 2026-05-25 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `JSerFeng`; linked commit authors: JSerFeng.
  Observation: Adds ESM lazy context loading; replies and patches preserve generic ensure handlers, CSS loading, and fetchPriority end to end, including multi-chunk callback argument correctness.
  Interaction: https://github.com/web-infra-dev/rspack/pull/14144#discussion_r3322982543 (2026-05-29T08:02:16Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/14144#discussion_r3323310201 (2026-05-29T09:05:50Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/14144#discussion_r3323328183 (2026-05-29T09:09:34Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/14144#discussion_r3323565455 (2026-05-29T09:56:08Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/14144#discussion_r3323687487 (2026-05-29T10:21:30Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/14144#discussion_r3323804804 (2026-05-29T10:46:47Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/15846; date 2026-09-23 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `JSerFeng`; linked commit authors: JSerFeng.
  Observation: Retains deferred-import dependency subgraphs in module factories instead of scope hoisting them eagerly; focused runtime regression checks evaluation timing and cache/name handling.

## CPunisher
Profile: https://github.com/CPunisher; API confirms login `CPunisher`, type `User`. Census: 80 merged PRs including reserved cases, 27 nonheld-out inline comments.
Ownership/evidence limitation: Two authored and two reviewed cases. Reuse/simplify guidance occurs on two reviewed PRs only; treat as bounded review guidance, not universal blocker. The request to construct a new targeted Emitter is an explicit exception to blanket reuse.
Bounded convention (multiple source kinds): first seek the existing location/diagnostic path; use a smaller purpose-specific path if the generic one incurs unrelated work (#12433/#13872). Validate compiler/backend upgrades with concrete regression behavior (#12747/#14352); not evidence that every upgrade has such a test.

- Source: https://github.com/web-infra-dev/rspack/pull/12433; date 2025-12-12 (PR creation), repo `web-infra-dev/rspack`, public, kind: review + diff + commits. PR author `SyMind`; linked commit authors: SyMind.
  Observation: Review asks to finish an unfinished branch, remove empty conditional, reuse byte_line_column_to_offset, and consider consolidating similar location calculations.
  Interaction: https://github.com/web-infra-dev/rspack/pull/12433#discussion_r2736391249 (2026-01-28T12:21:53Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12433#discussion_r2736474918 (2026-01-28T12:45:39Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12433#discussion_r2736486082 (2026-01-28T12:48:52Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12433#discussion_r2736488725 (2026-01-28T12:49:40Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12433#discussion_r2736501043 (2026-01-28T12:52:55Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/13872; date 2026-04-29 (PR creation), repo `web-infra-dev/rspack`, public, kind: review + diff + commits. PR author `Timeless0911`; linked commit authors: Timeless0911.
  Observation: Review asks to reuse diagnostic emitters, keep declaration output in a dedicated field, simplify path resolution and place declarations near use; separately asks to construct a targeted Emitter because the general self.print path unnecessarily generates sourcemaps.
  Interaction: https://github.com/web-infra-dev/rspack/pull/13872#discussion_r3159177563 (2026-04-29T07:10:15Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13872#discussion_r3159225662 (2026-04-29T07:21:14Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13872#discussion_r3159242736 (2026-04-29T07:25:05Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13872#discussion_r3159281479 (2026-04-29T07:32:22Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13872#discussion_r3159305939 (2026-04-29T07:37:45Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13872#discussion_r3159309746 (2026-04-29T07:38:35Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13872#discussion_r3159316304 (2026-04-29T07:39:58Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/12747; date 2026-01-15 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `CPunisher`; linked commit authors: CPunisher.
  Observation: Changes browser worker output from ESM to IIFE and adds browser React end-to-end coverage and WASM CI wiring for runtime format failures.

- Source: https://github.com/web-infra-dev/rspack/pull/14352; date 2026-06-11 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `CPunisher`; linked commit authors: CPunisher.
  Observation: Adds focused scope-hoisting regression for for-initializer binding identity; reports failure on swc_experimental 0.9.2 and success on 0.10.0.

## jerrykingxyz
Profile: https://github.com/jerrykingxyz; API confirms login `jerrykingxyz`, type `User`. Census: 46 merged PRs including reserved cases, 13 nonheld-out inline comments.
Ownership/evidence limitation: Three authored cache cases plus another author's readonly PR with verified reviewer commits. Distinguish four independent cases from eight comments on readonly. Cache correctness is supported across cases; avoid-E2E instruction is local, not a ban on E2E.
Supported operational convention: model cache validity independently of storage write policy and dependency category; invalidate affected downstream stages after load failure (#12902/#12274/#12805/#13608). Prefer the existing cache-case harness when it reproduces the issue (#12902, also authored fixtures #12274).

- Source: https://github.com/web-infra-dev/rspack/pull/12902; date 2026-01-30 (PR creation), repo `web-infra-dev/rspack`, public, kind: review + diff + commits. PR author `cellison-figma`; linked commit authors: cellison-figma, jerrykingxyz.
  Observation: Review separates readonly write policy from cache validity: changed build dependencies must invalidate reuse, readonly must not change cache version, invalid data must not be read or cleared. Supplies cacheCases fixture and asks to replace unnecessary E2E coverage, use Windows-compatible paths and rebuild before tests.
  Counterexample/limit: Mixed authorship; only linked comments and individually authored commits can establish personal choices.
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2752699778 (2026-02-02T06:05:42Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2752708024 (2026-02-02T06:09:27Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2752712491 (2026-02-02T06:11:09Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2752876240 (2026-02-02T07:08:33Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2753004846 (2026-02-02T07:50:00Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2753008198 (2026-02-02T07:51:07Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2762693108 (2026-02-04T08:00:53Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#discussion_r2767299583 (2026-02-05T06:08:48Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#pullrequestreview-3737469750 (2026-02-02T07:53:38Z, reviews).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#pullrequestreview-3771324596 (2026-02-09T05:52:28Z, reviews).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12902#issuecomment-3845935582 (2026-02-04T08:07:29Z, discussion).

- Source: https://github.com/web-infra-dev/rspack/pull/12274; date 2025-11-24 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `jerrykingxyz`; linked commit authors: jerrykingxyz.
  Observation: Tracks node_modules package.json in build dependencies despite stopping recursive resolution at that package boundary; extends cache regression fixtures.

- Source: https://github.com/web-infra-dev/rspack/pull/12805; date 2026-01-21 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `jerrykingxyz`; linked commit authors: jerrykingxyz.
  Observation: Separates snapshot strategies for file/context/missing/build dependency categories: directory existence and children-content tracking are not interchangeable.

- Source: https://github.com/web-infra-dev/rspack/pull/13608; date 2026-04-03 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `jerrykingxyz`; linked commit authors: jerrykingxyz.
  Observation: Resets failed persistent-cache scope and downstream incremental state so a full stage rebuild replaces invalid data.

## fansenze
Profile: https://github.com/fansenze; API confirms login `fansenze`, type `User`. Census: 9 merged PRs including reserved cases, 2 nonheld-out inline comments.
Ownership/evidence limitation: Three actual authored linter changes establish narrow lint-integration stewardship only. Two inline replies on one PR contain no rationale. No substantive review style or broad runtime ownership established; keep mode routing narrow.
Supported narrow convention: land linter upgrades with config/command migration, lockfile updates and required local fixes; inspect newly enabled rules and record intentional opt-outs (#12746/#13398/#13793). No general review persona supported.

- Source: https://github.com/web-infra-dev/rspack/pull/12746; date 2026-01-15 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `fansenze`; linked commit authors: fansenze.
  Observation: Updates Rslint dependency and lockfile, enables default-param-last while explicitly disabling incompatible newly supplied rules.

- Source: https://github.com/web-infra-dev/rspack/pull/13398; date 2026-03-18 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `fansenze`; linked commit authors: fansenze.
  Observation: Migrates Rslint JSON configuration to typed config, updates lint command and dependency/lockfile, preserves explicit project/ignore/rule choices; replies only say updated.
  Interaction: https://github.com/web-infra-dev/rspack/pull/13398#discussion_r2951219555 (2026-03-18T06:27:00Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/13398#discussion_r2951223136 (2026-03-18T06:28:06Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/13793; date 2026-04-22 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `fansenze`; linked commit authors: fansenze.
  Observation: Updates Rslint and lockfile with necessary source fixes for new diagnostics and an explicit prefer-spread opt-out.

## quininer
Profile: https://github.com/quininer; API confirms login `quininer`, type `User`. Census: 7 merged PRs including reserved cases, 10 nonheld-out inline comments.
Ownership/evidence limitation: Two authored changes and two reviews; safety guidance has authored and reviewed support. Shorter-lock guidance is one independent PR, and federation cleanup lifecycle is explicitly uncertain. Do not infer a no-unsafe policy.
Supported safety convention (authored+reviewed): use safe iteration and type-checked/disjoint mutation when possible; do not treat black_box as aliasing proof (#12006/#12046). Consider minimum guard lifetime in phase-scoped mutable state (#12067, tentative as one case).

- Source: https://github.com/web-infra-dev/rspack/pull/12006; date 2025-10-27 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `quininer`; linked commit authors: quininer.
  Observation: Removes redundant unsafe Send/Sync impls and replaces unchecked indexing with a safe reverse iterator.

- Source: https://github.com/web-infra-dev/rspack/pull/12046; date 2025-10-30 (PR creation), repo `web-infra-dev/rspack`, public, kind: review + diff + commits. PR author `JSerFeng`; linked commit authors: JSerFeng.
  Observation: Review rejects shared-reference-to-mutable-reference cast via black_box as unsound; proposes UnsafeCell or preferably proving disjoint mutable access/type-checked synchronization.
  Interaction: https://github.com/web-infra-dev/rspack/pull/12046#discussion_r2478326155 (2025-10-30T14:26:39Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12046#discussion_r2478355029 (2025-10-30T14:34:37Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/12067; date 2025-11-03 (PR creation), repo `web-infra-dev/rspack`, public, kind: review + diff + commits. PR author `JSerFeng`; linked commit authors: JSerFeng.
  Observation: Review recommends dropping the mutation guard after assignment to reduce its scope, including for generated-code quality; underlying author change replaces once-only state with rebuild-compatible AtomicRefCell.
  Interaction: https://github.com/web-infra-dev/rspack/pull/12067#discussion_r2485659830 (2025-11-03T08:29:51Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/12067#discussion_r2485682059 (2025-11-03T08:39:08Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/11929; date 2025-10-20 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `quininer`; linked commit authors: quininer.
  Observation: Adds clear_cache removal for compilation-scoped federation hooks map to stop lifetime retention; author explicitly expresses uncertainty about the lifecycle point.

## 2heal1
Profile: https://github.com/2heal1; API confirms login `2heal1`, type `User`. Census: 21 merged PRs including reserved cases, 6 nonheld-out inline comments.
Ownership/evidence limitation: Include as inferred MF specialist: repeated substantive manifest implementation and fixes plus a later review diagnosing and verifying runtime fallback behavior. This is observed ownership, not a declared governance role or blanket merge authority.
Supported specialist convention: exercise MF manifest data against shared/remote/exposed modules and entry-only assets, and use runtime execution to validate loadable fallbacks (#11846/#12399/#12836 plus #14892 review). Do not extrapolate performance motivation to a blanket Rust-over-JS requirement.

- Source: https://github.com/web-infra-dev/rspack/pull/11846; date 2025-10-11 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `2heal1`; linked commit authors: 2heal1.
  Observation: Authors Rust-based Module Federation manifest generation to avoid costly JavaScript stats.toJson path; adds manifest cases; agrees default-off until v2.
  Interaction: https://github.com/web-infra-dev/rspack/pull/11846#discussion_r2458495140 (2025-10-24T02:41:41Z, inline).

- Source: https://github.com/web-infra-dev/rspack/pull/12399; date 2025-12-09 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `2heal1`; linked commit authors: 2heal1.
  Observation: Extends MF manifest metadata for configured remotes, scoped shared packages, singleton/version configuration, file tracking and matching fixtures.

- Source: https://github.com/web-infra-dev/rspack/pull/12836; date 2026-01-26 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `2heal1`; linked commit authors: 2heal1.
  Observation: Fixes MF manifest exposure asset sets to exclude host-entry/vendor/async chunks; factors helpers and adds entry-filter fixture.

- Source: https://github.com/web-infra-dev/rspack/pull/14892; date 2026-07-21 (PR creation), repo `web-infra-dev/rspack`, public, kind: review + diff + commits. PR author `jcampalo`; linked commit authors: jcampalo.
  Observation: Reviews another author's ESM shared fallback fix: demands actual emitted-container import/init/get execution rather than emitted-file/source assertions; locally reproduces missing ensure handlers under webpack runtime mode and identifies initialization dependency.
  Interaction: https://github.com/web-infra-dev/rspack/pull/14892#discussion_r3871527582 (2026-08-27T12:07:19Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/14892#issuecomment-5438899371 (2026-08-27T12:11:13Z, discussion).

## colinaaa
Profile: https://github.com/colinaaa; API confirms login `colinaaa`, type `User`. Census: 6 merged PRs including reserved cases, 9 nonheld-out inline comments.
Ownership/evidence limitation: Keep uncertain core candidate: six merged PRs across source maps/types/test tooling/compatibility, with nine training comments concentrated on own #11814. No sustained cross-author review/triage evidence in this sample.
Evidence only; no hard personal rules. Compatibility cases show checking upstream semantics and documenting current gaps, but core role remains uncertain.

- Source: https://github.com/web-infra-dev/rspack/pull/11814; date 2025-10-06 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `colinaaa`; linked commit authors: colinaaa.
  Observation: Ports extractSourceMap and source-map/virtual-module tests; replies compare webpack snapshots and acknowledge HTTP URL extraction remains follow-up, with targeted assertions replacing divergent full snapshots.
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2405437275 (2025-10-06T09:11:57Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2405439788 (2025-10-06T09:12:58Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2405444251 (2025-10-06T09:14:42Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2423331918 (2025-10-12T04:39:17Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2423331987 (2025-10-12T04:39:46Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2425732684 (2025-10-13T09:43:29Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2425741372 (2025-10-13T09:47:10Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2425973715 (2025-10-13T11:05:19Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#discussion_r2426084721 (2025-10-13T11:40:37Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/11814#issuecomment-3393493642 (2025-10-11T16:46:46Z, discussion).

- Source: https://github.com/web-infra-dev/rspack/pull/13048; date 2026-02-12 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `colinaaa`; linked commit authors: colinaaa.
  Observation: Fixes published lite-tapable type generation rather than runtime bundling; narrow packaging maintenance evidence.

- Source: https://github.com/web-infra-dev/rspack/pull/14361; date 2026-06-11 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `colinaaa`; linked commit authors: colinaaa.
  Observation: Ports upstream webpack overrideStrict context-module behavior with adapted upstream config case; PR explains earlier divergence was blocked until upstream landed and records binding build before regression run.

## kakiuwang-ui
Profile: https://github.com/kakiuwang-ui; API confirms login `kakiuwang-ui`, type `User`. Census: 10 merged PRs including reserved cases, 4 nonheld-out inline comments.
Ownership/evidence limitation: Keep emerging/uncertain core candidate: nine substantial recent API/binding PRs, but concentrated Aug-Sep 2026 and mostly own-PR replies; no broad cross-author review evidence. #15305 and #15372 include LingyuCoder; #15305 also has unlinked author(s).
Evidence only; no hard personal rules. Recent binding/API changes use repository hooks, rebuild reproductions and bilingual docs; repository requirements must not become personal preference claims.

- Source: https://github.com/web-infra-dev/rspack/pull/15454; date 2026-09-04 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `kakiuwang-ui`; linked commit authors: kakiuwang-ui.
  Observation: Guards stale/mid-pass JS Compilation module-graph access and reproduces abort in an in-repository three-build watch case rather than requiring external Next integration.

- Source: https://github.com/web-infra-dev/rspack/pull/15305; date 2026-08-23 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `kakiuwang-ui`; linked commit authors: LingyuCoder, None, kakiuwang-ui.
  Observation: Adds runtimeTemplate using webpack/Rust counterparts, environment tests and English/Chinese API docs; replies fix webworker neutral-platform classification and retain webpack-compatible behavior. Mixed collaborators: do not attribute entire diff solely to author.
  Counterexample/limit: Mixed authorship; only linked comments and individually authored commits can establish personal choices.
  Interaction: https://github.com/web-infra-dev/rspack/pull/15305#discussion_r3841310706 (2026-08-24T07:06:06Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/15305#discussion_r3841310883 (2026-08-24T07:06:09Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/15305#discussion_r3841311150 (2026-08-24T07:06:11Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/15305#issuecomment-5391917249 (2026-08-24T07:15:28Z, discussion).

- Source: https://github.com/web-infra-dev/rspack/pull/15372; date 2026-08-28 (PR creation), repo `web-infra-dev/rspack`, public, kind: authored PR + diff + commits. PR author `kakiuwang-ui`; linked commit authors: LingyuCoder, kakiuwang-ui.
  Observation: Adds afterOptimizeChunkIds hook following existing Rust/binding/JS registration pattern, with skip-when-untapped, rebuild tests and bilingual API docs; includes collaborator commit.
  Counterexample/limit: Mixed authorship; only linked comments and individually authored commits can establish personal choices.
  Interaction: https://github.com/web-infra-dev/rspack/pull/15372#discussion_r3906480868 (2026-09-01T17:03:05Z, inline).
  Interaction: https://github.com/web-infra-dev/rspack/pull/15372#issuecomment-5472773372 (2026-08-31T02:00:02Z, discussion).

## Synthesis boundaries
No unanimity claim. ahabhgk and jerrykingxyz provide complementary cache-correctness evidence; JSerFeng supplies ESM/output semantics; CPunisher compiler/browser and diagnostic review; quininer Rust safety; fansenze lint maintenance; 2heal1 Module Federation. Observed test choice depends on failure: jerrykingxyz rejects an unnecessary E2E in favor of cacheCases, whereas CPunisher adds browser E2E for browser worker execution. This is contextual routing, not disagreement. Reuse is conditional: CPunisher requests reuse of diagnostics but a dedicated cheaper Emitter instead of heavy self.print.
Unsupported: universal coding-style preferences for all members, personal motives, complete release authority, independent triage coverage and exhaustive discussions coverage, ongoing refresh. fansenze has insufficient substantive reviewed PRs in supplied inline corpus for review fidelity evaluation. quininer reviewed cases #12046 and #12067 were training, not holds; seek fresh holdouts if evaluating.

## Supplemental discussion evidence

- JSerFeng: https://github.com/web-infra-dev/rspack/discussions/15802 (public web-infra-dev/rspack, author JSerFeng, RFC, date recorded in raw discussions.json). Explains module-identity/evaluation-order limits, request-count/duplicate-byte tradeoffs, bounded candidate-discovery complexity and production/development defaults. Supports making optimization tradeoffs explicit and preserving existing splitChunks constraints, alongside authored semantic regression cases. Counterexample: larger search depth does not guarantee smaller output; never turn the proposal into "maximize deduplication".
- JSerFeng: https://github.com/web-infra-dev/rspack/discussions/12653 (2026-01-07, public, authored RFC); replies https://github.com/web-infra-dev/rspack/discussions/12653#discussioncomment-16585788 (2026-04-16). Separates parse collection from finishModules analysis when cross-module context is required, and explains limitations of purity analysis with a concrete playground example. Counterexample: RFC/reply describes historical restrictions; do not encode the then-current function-declaration-only limitation as today's invariant.
- jerrykingxyz: https://github.com/web-infra-dev/rspack/discussions/11965#discussioncomment-14747306 (2025-10-22, public, ownership attribution by chenjiahan) explicitly identifies portable/remote-cache research responsibility. https://github.com/web-infra-dev/rspack/discussions/11965#discussioncomment-14987416 (2025-11-17, public, jerrykingxyz reply) links portable-cache RFC #12218. Strengthens cache ownership, not proof of implementation or general personal design rule. Linked #12218 was not fetched by this worker.

Supplemental source coverage: three discussion topics, complete selected bodies and returned comment/reply pages inspected. Parent enumerated metadata; worker did not repeat the census. Raw selected records in discussions.json.
