---
name: bump-rspack-swc
description: Upgrade the SWC Rust dependencies used by the Rspack repository, regenerate workspace version code, analyze the swc_core tag-to-tag changelog for breaking and Rspack-relevant changes, correlate fixes with Rspack issues, validate the upgrade, and commit, push, and open the GitHub pull request. Use when asked to bump, update, or upgrade swc_core or its aligned SWC crate versions in Rspack and prepare or submit the resulting PR.
---

# Upgrade Rspack SWC

Perform the upgrade end to end. Keep the patch focused, preserve unrelated work, and base every changelog claim on the SWC tag range.

## Guardrails

- Confirm the checkout is `web-infra-dev/rspack` and read the applicable `AGENTS.md` instructions before editing.
- Inspect `git status --short`, the current branch, and remotes first. Never overwrite, discard, stage, or commit unrelated user changes. Stop for direction if unrelated changes overlap the files required by the upgrade.
- Use exact, stable, non-yanked crate versions. Ignore prereleases unless the user explicitly requests one.
- Treat the root `Cargo.toml` comment `Must be pinned with the same swc versions` as an invariant. Keep SWC direct dependencies on one compatible release set; do not blindly bump only `swc_core`.
- Treat every `swc_experimental_*` crate as independent and out of scope. Never query, bump, or otherwise change its manifest or lockfile version as part of this workflow.
- Keep existing `default-features`, feature lists, and unrelated dependency versions unchanged unless the new SWC release requires an adaptation.
- Do not edit `crates/rspack_workspace/src/generated.rs` by hand. Generate it with `cargo codegen`.
- Use official sources for release data: crates.io and `swc-project/swc` tags, commits, and pull requests.
- Do not claim an SWC change fixes a Rspack issue without matching the failure mode and validating the fix when practical. Use “related” or “may fix” for weaker evidence.
- Use `pnpm run build:binding:dev` as the only validation command. Do not run tests or lint unless the user explicitly requests them.
- Do not create the PR until the intended diff and required validation have been reviewed. If validation is incomplete or an important risk remains, explain it and open a draft only when the user still wants a PR.

## 1. Establish the version range

1. Read the SWC dependency block in the repository root `Cargo.toml` and record the exact current `swc_core` version as `OLD_VERSION`.
2. Query crates.io for the newest stable, non-yanked `swc_core` release and record it as `NEW_VERSION`. Cross-check that the tag `swc_core@NEW_VERSION` exists in `https://github.com/swc-project/swc`.
3. Stop with a no-op report if `OLD_VERSION` already equals `NEW_VERSION`.
4. Enumerate the in-scope direct root dependencies named `swc` or beginning with `swc_`. At `swc_core@NEW_VERSION`, inspect the SWC workspace and crate manifests to identify their release-compatible versions. Also check crates.io for compatible patch releases published after the tag.
5. Update only those in-scope entries in the root `Cargo.toml`. If the upgrade cannot resolve without changing an excluded dependency, stop and report the incompatibility instead of expanding scope.
6. Refresh `Cargo.lock` through Cargo resolution, using a precise `swc_core` update where needed. Inspect resolution errors rather than loosening pins or features speculatively.
7. Check the resolved SWC graph for unintended duplicate major versions, especially `swc_common`, AST, parser, transform, minifier, plugin, and HTML crates. Resolve incompatible direct pins before proceeding.

Preserve full semantic versions for Cargo and tag operations. For the PR title labels, follow the recent Rspack SWC bump convention: use major-only labels for a major-to-major bump, otherwise use the shortest unambiguous semantic versions.

## 2. Regenerate workspace code

Run from the repository root:

```bash
cargo codegen
```

Verify that `crates/rspack_workspace/src/generated.rs` contains `NEW_VERSION` in `rspack_swc_core_version()` and that no unrelated generated values changed.

## 3. Build the tag-to-tag changelog

Use these exact refs:

```text
swc_core@OLD_VERSION...swc_core@NEW_VERSION
```

Create the canonical comparison link:

```text
https://github.com/swc-project/swc/compare/swc_core%40OLD_VERSION...swc_core%40NEW_VERSION
```

Collect the complete commit list through the GitHub compare API or the exact tag refs in a temporary/local SWC checkout. Paginate and de-duplicate by SHA. If GitHub truncates a large comparison, split it across consecutive `swc_core@...` tags and merge the results. Do not substitute dates, branches, or release-page prose for the tag range.

For commits that reference a pull request, inspect that PR’s title, body, labels, and relevant files. Drop release bookkeeping, dependency noise, and merges with no user-visible impact. Classify the remaining changes as:

- breaking or required Rspack adaptations;
- parser, AST, resolver, codegen, source-map, transform, React compiler, TypeScript, minifier, plugin, HTML, performance, or correctness changes relevant to Rspack;
- small fixes that can be combined into one concise sentence.

For every breaking or notable item, link the upstream PR or commit and explain the concrete Rspack surface it affects. Search the Rspack checkout for changed SWC API names and behavior so the summary reflects actual integration points instead of repeating upstream titles.

Search open and recent Rspack issues using the symptoms, syntax, transform names, and error text from notable SWC fixes. Read candidate issues and confirm the behavior matches from the available issue details, upstream fix, and local integration code. Do not run tests as part of this workflow. Record verified issue links for the PR; distinguish confirmed fixes from plausible connections.

## 4. Adapt and validate

Fix compilation changes caused by the new SWC release with the smallest compatible patch. Validate the upgrade by running only:

```bash
pnpm run build:binding:dev
```

Do not run unit tests, integration tests, end-to-end tests, lint, or additional validation commands unless the user explicitly requests them. If the build fails because of the SWC upgrade, make the smallest necessary compatibility fix and rerun the same build command. If an environment failure prevents the build, capture the exact failure; never present it as passing.

Review the final diff. Expected files are normally root `Cargo.toml`, `Cargo.lock`, and `crates/rspack_workspace/src/generated.rs`, plus only necessary compatibility changes. Confirm the generated version is correct and inspect the resolved SWC versions one final time.

## 5. Commit and open the PR

Use this exact title shape, retaining the backticks:

```text
chore: bump `swc_core` from FROM_LABEL to TO_LABEL
```

Use the same subject for the commit unless repository history indicates a necessary commit-only variation. Stage only intended paths, review the staged diff, commit, push the current feature branch, and open the PR against `main`.

Build the PR description from `.github/PULL_REQUEST_TEMPLATE.md`. Keep it brief and evidence-based:

```markdown
## Summary

Bumps `swc_core` from `OLD_VERSION` to `NEW_VERSION` and aligns the compatible SWC dependency pins.

[Upstream tag comparison](COMPARE_URL)

- **Rspack-relevant / breaking:** <only the important changes and required adaptations, with upstream links>
- **Fixes:** <group small fixes into one sentence; call out verified Rspack bugs separately>

## Related links

<Use `Fixes #NNNN` only for verified fixes; otherwise use `Related to #NNNN` or `N/A`.>

## Checklist

- [x] Tests updated (or not required).
- [x] Documentation updated (or not required).

## Validation

- `<exact command>`
```

Include only useful changelog detail in the PR body; do not dump the raw commit list. Mark a checklist item complete only when true, and state why tests or docs are not required when that is not obvious. After creation, report the PR URL, exact version range, changed files, validation results, and any remaining risks.
