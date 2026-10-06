# Scope and collection record

This mode covers public `web-infra-dev/rspack` evidence dated 2025-10-06 through 2026-10-06. Collection took place on 2026-10-06. The default branch was `main`, and repository requirements were checked at `682a90cdfc2455cb0974e369578fd02c8c93dd8e`. The intended jobs are code writing and review. No other repository was mined.

## Discovery and sampling

- Enumerated 2,734 merged PRs using 14 nonoverlapping date partitions and paginated GitHub search. Each partition stayed below the 1,000-result cap. A temporary per-minute search rate limit interrupted the final partitions; they were resumed successfully.
- Enumerated 5,680 inline comments with creation dates in the window through the paginated repository endpoint. Bot records and Copilot were excluded from human evidence. Replies on an author's own PR do not establish independent review ownership.
- Enumerated all 379 discussion metadata records, including older discussions. Read selected in-window bodies and dated replies only as evidence; the metadata total is not a body-coverage claim.
- Checked the published team and CODEOWNERS against authored changes, dated reviews, and selected issue activity. Sparse-member discovery covered 266 PR and issue records with dated review/comment metadata and exhausted their returned pages.
- Inspected representative training PRs separately for each person. Selected details include patches, linked commit authors, reviews, inline context, and replies. This is a bounded evidence sample, not an exhaustive reading of every conversation or commit.
- Reserved 12 PR conversations before training-body inspection. All mining workers excluded the entire reserved set. See [validation](validation.md) for historical commits, results, and limitations.

Authored merged counts use the exact PR author, not the search qualifier alone. For example, `author:SoonIter` returned two PRs actually authored by `copilot-swe-agent`; those do not count as SoonIter-authored changes. A PR author, commit author, coauthor, and reviewer are different roles. Check the individual ledgers before attributing an implementation.

## Source boundaries

The output and every cited evidence repository are public. Linked external projects provide context only and were not crawled. Profiles establish account identity, not personality or private motives. Unexplained approvals, emoji replies, and boilerplate do not establish a merge standard. Issue involvement and updated timestamps alone do not establish dated participation.

Repository instructions are authoritative for work in this checkout. An observed preference is conditional and cannot override those instructions. A proposal or an unmerged PR is not an adopted API contract. No rule claims that every roster member agrees with it.

## Refresh

Start from this cutoff and recheck the current published team, CODEOWNERS, and dated activity. Reassess the sparse and emerging accounts in the roster rather than treating this membership as permanent. Retain supported rules, record counterexamples, and remove outdated ones. Reserve fresh reviewed PRs before mining. Record new coverage and evaluation limits in these references. Refreshing this mode does not authorize posting reviews, impersonating maintainers, or scheduling a crawler.
