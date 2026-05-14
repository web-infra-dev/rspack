---
name: create-draft-release-notes
description: Create or update draft GitHub releases for the current project's main GitHub repository, then organize GitHub-generated release notes into user-friendly sections without rewriting release note items. Use for preparing, formatting, categorizing, creating, or updating GitHub release notes or draft releases.
---

# Create Draft Release Notes

## Overview

Create a GitHub draft release, organize the generated notes by conventional commit type, and save the organized body back to the draft. Preserve each release note item exactly; only split accidentally joined bullets, move bullets into sections, and adjust headings.

## Draft Release Workflow

Input: a release tag/title such as `v2.0.6`. If title and tag differ, ask for the tag.

1. Resolve `repo` as `<owner>/<repo>`.
   Prefer an explicit repo from the user. Otherwise infer the current project's main GitHub repository from project metadata or the current GitHub remote. For npm projects, `package.json` `repository` is a useful signal; in monorepos, inspect the package or project being released rather than assuming the workspace root. Ignore subdirectory metadata such as `repository.directory` because GitHub releases are repository-level. If the repo is ambiguous, ask.

2. Set variables:

   ```bash
   repo="<owner>/<repo>"
   release_tag="v2.0.6"
   release_title="$release_tag"
   ```

3. Verify access and ensure the release does not already exist:

   ```bash
   gh auth status
   gh repo view "$repo" --json nameWithOwner --jq '.nameWithOwner'
   gh release view "$release_tag" -R "$repo" --json tagName,isDraft,url
   ```

   If the release exists, stop unless the user explicitly asked to update that draft.

4. Infer the previous tag:

   ```bash
   previous_tag="$(gh release list -R "$repo" --exclude-drafts --exclude-pre-releases --limit 1 --json tagName --jq '.[0].tagName')"
   gh release list -R "$repo" --exclude-drafts --exclude-pre-releases --limit 5
   ```

   Ask for confirmation if the previous tag is missing, surprising, or part of a non-standard range.

5. Before creating anything, state the repo and range: `previous_tag -> release_tag`. If the user did not explicitly ask to create the draft in this turn, ask for confirmation.

6. Create the draft with GitHub-generated notes:

   ```bash
   gh release create "$release_tag" -R "$repo" --draft --generate-notes --notes-start-tag "$previous_tag" --title "$release_title"
   ```

   Add `--verify-tag` when the release must use an existing remote tag.

7. Organize and save the draft body:

   ```bash
   tmp_dir="$(mktemp -d)"
   gh release view "$release_tag" -R "$repo" --json body --jq '.body' > "$tmp_dir/generated.md"
   node .agents/skills/create-draft-release-notes/scripts/create-draft-release-notes.mjs "$tmp_dir/generated.md" > "$tmp_dir/organized.md"
   gh release edit "$release_tag" -R "$repo" --draft --title "$release_title" --notes-file "$tmp_dir/organized.md"
   ```

8. Return the draft URL:

   ```bash
   gh release view "$release_tag" -R "$repo" --json url --jq '.url'
   ```

## Markdown-Only Workflow

Use this when the user provides generated release note Markdown and only wants it organized:

```bash
node .agents/skills/create-draft-release-notes/scripts/create-draft-release-notes.mjs release-notes.md
```

Omit the file path to read from stdin. Review that every original item still appears once and non-item sections remain.

## Categories

Emit non-empty sections in this order:

1. `### Breaking Changes 🍭`
2. `### New Features 🎉`
3. `### Performance 🚀`
4. `### Bug Fixes 🐞`
5. `### Refactor 🔨`
6. `### Document 📖`
7. `### Other Changes`

Classify by the item prefix:

- Breaking Changes: `type!:` or `type(scope)!:`, plus `breaking:` / `break:`.
- New Features: `feat:` / `feat(scope):`, plus `feature:`.
- Performance: `perf:`.
- Bug Fixes: `fix:`.
- Refactor: `refactor:`.
- Document: `docs:` / `docs(scope):`, plus `doc:`.
- Other Changes: everything else.

Keep each category in generated top-to-bottom order.

## Preservation Rules

- Do not rewrite bullet text, authors, URLs, PR numbers, package names, scopes, punctuation, or casing.
- Do not drop comments, `**Full Changelog**`, or other non-item sections.
- Do not add commentary to the release note itself.
- Do not emit empty category sections.

## Resources

- `scripts/create-draft-release-notes.mjs`: deterministic formatter for generated release note Markdown.
