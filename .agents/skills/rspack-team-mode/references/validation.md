# Validation and held-out results

## Artifact checks

The skill-creator validator passes. Local reference targets and anchors were checked, and prose was reviewed with pstack's unslop guidance. Evidence links were retrieved during collection; public GitHub source visibility was verified. No unrelated mode name was overwritten. Automatic discovery remains enabled with a specific Rspack-team trigger.

This change contains only skill instructions, metadata, and evidence references. It does not change bundler code or APIs. No Rspack build or runtime tests were run; the skill validator does not consume compiled bundler artifacts.

## Blinded review smoke test

One reviewed PR per available selected reviewer was reserved, 12 total, before training-body inspection. Selection required a merged PR by another author and an inline comment of at least 90 characters; this metadata heuristic did not guarantee representative review content. Other roster members had no reserved case under this selection, which does not prove they have no substantive reviews.

Three agents evaluated four cases each with the draft skill and historical diffs. Their predictions were saved before actual comments were exposed. The evaluators had participated in mining, but every held-out conversation was excluded from their training. This was a blinded holdout pass, not a fresh-context or controlled baseline comparison.

GitHub review records supplied the reviewed commit. Git comparisons reconstructed complete diffs against the associated merge base, replacing API patches that were absent or truncated. Evaluators received those diffs and commit metadata, not later fixes or conversations. Surrounding historical source outside the diff and test execution were unavailable. The two largest cases were explicitly partial inspections.

No case demonstrated a full substantive-ask match. Two predictions raised related concerns with different specific requests. This does not establish reviewer fidelity. Predicted extra findings were not executed or independently confirmed, and are not evidence that the maintainers missed bugs. The table records comparison rather than treating absence of a blocker as approval.

| Account and held-out case | Actual selected review | Blind prediction and comparison |
| --- | --- | --- |
| [hardfist, #15467](https://github.com/web-infra-dev/rspack/pull/15467#discussion_r3946625155) | Diagnostic deduplication and fragile English-message retry/filter behavior; explicit request-changes text. | Raised platform withdrawal instead. Miss; 1.1 MB migration only partly inspected. |
| [chenjiahan, #15429](https://github.com/web-infra-dev/rspack/pull/15429#discussion_r3911175980) | Correct afterOptimizeModules from SyncBailHook to SyncHook in both languages. | No established defect from diff alone; declaration outside the input. Miss. |
| [LingyuCoder, #15361](https://github.com/web-infra-dev/rspack/pull/15361#discussion_r3892301166) | Use webpack lazy fallback in the context-parser path. | Suggested invalid-mode execution coverage elsewhere. Related fallback theme, different ask. |
| [ahabhgk, #13703](https://github.com/web-infra-dev/rspack/pull/13703#discussion_r3086327799) | Remove experimental import.meta.resolve claim from release post. | Suggested Node version wording. Miss; release-post case poorly matches cache-focused training. |
| [JSerFeng, #14917](https://github.com/web-infra-dev/rspack/pull/14917#discussion_r3642803210) | Remove default trait implementation so implementers explicitly assess runtime_module_variables. | Asked about generic provider registration and runtime validation. Miss. |
| [stormslowly, #12417](https://github.com/web-infra-dev/rspack/pull/12417#discussion_r2664056075) | Do not implicitly pin a PnP manifest and break references to other Yarn projects. | Questioned cwd-based manifest discovery and tests. Related selection contract, different cross-project rationale. |
| [SyMind, #14852](https://github.com/web-infra-dev/rspack/pull/14852#discussion_r3764905401) | Reuse normal_module.source and keep replacement bookkeeping in concatenation instead of new source APIs. | Raised escaped-identifier identity issue. Miss; large diff only partly inspected, finding not executed. |
| [intellild, #15433](https://github.com/web-infra-dev/rspack/pull/15433#discussion_r3994557358) | Reuse private Raw binding types to respect per-PR binary-size constraints. | Raised hook composition and loader-cache coverage. Miss. |
| [CPunisher, #14037](https://github.com/web-infra-dev/rspack/pull/14037#discussion_r3256977484) | Move only the required DTS fields into queue/output to avoid cloning unrelated data. | Suggested path-helper reuse and transitive/watch coverage. Miss. |
| [jerrykingxyz, #13731](https://github.com/web-infra-dev/rspack/pull/13731#discussion_r3093021804) | Avoid direct Git dependency; consider an rkyv feature inside rspack_sources. | Raised corrupt-cache recovery and roundtrip output checks. Miss. |
| [Timeless0911, #13861](https://github.com/web-infra-dev/rspack/pull/13861#discussion_r3216161794) | Optionally add examples for the new external type later. | Raised target semantics and Node execution coverage. Miss; did not predict the optional documentation request. |
| [fi3ework, #15689](https://github.com/web-infra-dev/rspack/pull/15689#discussion_r4013706798) | Consider file_stem and an mjs/cjs fixture for extension-consistent adjacent mocks. | Suggested root precedence under aliases/symlinks. Miss. |

Only hardfist's selected review body contains an explicit request-changes verdict, despite its API state being COMMENTED. The remaining selected records have no explicit verdict. No approval agreement or aggregate verdict score is claimed. Timeless0911's suggestion was optional, and ahabhgk's record explains an editorial change; neither supports inventing a blocking standard. The hardfist review describes consolidated committee feedback and is account activity rather than proof of unaided personal judgment.

## Historical commits

| Account | Reviewed commit |
| --- | --- |
| hardfist | `0346f7424f98ceda5bf4c06c15495cc32cc972bb` |
| chenjiahan | `6520b376f52039d045e851a784d4ef9edeaeb726` |
| LingyuCoder | `97f0f29562ac8634d0e679e05684a61f9df347dc` |
| ahabhgk | `90415c7eaf6ff474828e89da5d09cc97d30b8294` |
| JSerFeng | `caf9ceb2cadfa6cb0c004a2272926c3464c75b00` |
| stormslowly | `32e920abb9069d01450e807cce906f27a8568869` |
| SyMind | `67ebe01f9b5fdba21b337ef6765c8b38f5ca16f2` |
| intellild | `cd3200c0021bf64e7063f2ddb736e3f16c3a6159` |
| CPunisher | `81e7ee3bc7ae2bb587c92b41e4d382fc97dcf078` |
| jerrykingxyz | `2b8339eeccf2367ec66b6d99ff9c8f6acb8da4e9` |
| Timeless0911 | `4ba3b9606a0ee5102aa2787090ba558234202217` |
| fi3ework | `2ec6e612a91bde270435fbfcdbefbf358d7dc9b0` |

## Coverage that remains unverified

- 9aoy, yifancong, SoonIter, elecmonkey, ScriptedAlchemy, fansenze, quininer, and 2heal1 have no independent held-out review result in this run. The sparse and uncertain roster entries have no such result either.
- Code-writing conventions were checked against actual authored patches and account-linked commits during mining. No independent code-generation exercise or historical build/test reproduction was performed. Review evaluation cannot validate code-writing fidelity.
- Issue triage and release decisions were sampled. Labels and closures were not attributed without actor evidence. The ledgers give per-batch omissions, including one missing PR reply fetch and capped issue discovery samples.
- The training batches inspected 22, 33, and 28 PR records respectively; these counts describe batch coverage, not an exhaustive crawl of all 2,734 merged PRs. Multiple account sections can discuss the same PR.
- Review expectations from unmerged proposals remain proposals; authored work with bot or collaborator contributions is labeled. No claim of unanimous team preference is supported.

No operational convention was added from the held-out answers. The only post-evaluation change to the entrypoint disclosed these limits. Future fidelity claims need fresh cases, surrounding historical context, separate code-writing evaluation, and preserved misses. Use this mode as scoped engineering guidance, not a substitute for current maintainer review.
