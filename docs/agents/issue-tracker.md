# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

External contributor PRs are also a triage surface. `/triage` should include open PRs whose author association is `CONTRIBUTOR`, `FIRST_TIME_CONTRIBUTOR`, or `NONE`, and should leave `OWNER`, `MEMBER`, and `COLLABORATOR` PRs alone.

When a skill says "publish to the issue tracker", create a GitHub issue.

When a skill says "fetch the relevant ticket", run `gh issue view <number> --comments`.

Use `gh issue` for issues and `gh pr` for PRs. GitHub shares one number space across issues and PRs, so resolve ambiguous `#<number>` references with `gh pr view <number>` first, then fall back to `gh issue view <number>`.
