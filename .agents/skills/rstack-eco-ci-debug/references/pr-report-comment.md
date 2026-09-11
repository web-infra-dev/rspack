# PR Report Comment Tool

Use this tool to comment on a merged source PR only when the eco-ci failure is strictly attributed to that PR. The source PR can be either:

- a PR in the selected ecosystem that changed the tested upstream artifact, or
- a downstream project PR that changed the project under test.

## Guardrails

- Do not comment if attribution is ambiguous, only temporal, or based only on a surface green-to-red pivot.
- Do not comment if a different downstream PR, different dependency bump, release window, flaky network issue, or changed failure signature is still a plausible cause.
- Do not comment on a selected-upstream surface pivot if the actual source is a downstream PR. Comment on the downstream PR instead, and explicitly say the upstream pivot was ruled out.
- Do not comment when the same failure signature appeared before the candidate PR, unless you have evidence that the PR made the flaky failure deterministic or changed its signature.
- Do not comment on config-gated PRs until generated config evidence proves the failing case enables the relevant option or path. For Rsbuild-based suites, read [rsbuild-config-debug.md](rsbuild-config-debug.md) when the hypothesis depends on Rsbuild/Rspack config.
- Verify the PR is merged before commenting.
- Ask the user for explicit approval before posting. A draft is safe; an actual GitHub comment is not.
- Include the marker at the beginning of the comment:

```text
<agent: daily-job rstack ecosystem-ci>
```

## Required Evidence Before Commenting

Collect and state these facts first:

- The failing suite name.
- The eco-ci run URL or run id.
- The selected ecosystem and tested upstream commit.
- The failure signature from GitHub Actions logs.
- The source repository and source PR number to comment on.
- The first bad commit or PR, with a visible success-to-failure pivot or equivalent canary bisect proof.
- A check that the same signature was not already known as flaky or pre-existing.
- For config-gated hypotheses, evidence from generated downstream config that the relevant option or path is actually enabled.
- For selected-upstream PR comments, why downstream changes and other plausible causes were ruled out.
- For downstream PR comments, why the visible upstream pivot is only a surface attribution and why the downstream PR is the actual source.

If any item is missing, do not post. Continue investigation or provide a draft-only note.

## Comment Workflow

1. Check PR metadata:

```bash
gh pr view <source-pr-number> --repo <source-owner/repo> --json number,title,state,mergedAt,author,url,headRefOid,mergeCommit
```

2. Refuse to post if `mergedAt` is empty.
3. Prepare a concise English comment with:
   - The required marker.
   - A descriptive heading naming the `<ecosystem>/<suite>` pair.
   - A visible attribution status and one-sentence impact summary.
   - A short mechanical explanation of why this PR is the source.
   - The preferred owner, fix direction, and smallest useful verification.
   - Detailed run, pivot, flaky, and config evidence inside a collapsed section so the conclusion remains easy to scan.
   - A precise distinction between "introduced a product regression" and "exposed an environment or downstream incompatibility" when ownership differs from the source PR.
   - Only conclusions supported by the collected evidence. In a draft, label missing values and checks as placeholders; for a real post, any missing required evidence still blocks sending.
4. Ask the user to approve posting.
5. Post only after approval:

```bash
gh pr comment <source-pr-number> --repo <source-owner/repo> --body-file <comment-file>
```

## Comment Template

Keep the conclusion and next action visible without expanding anything. Put supporting proof in `<details>` so maintainers can audit the attribution without making the default view noisy. Do not convert a required-but-missing check into a statement that the check passed.

```md
<agent: daily-job rstack ecosystem-ci>

### Ecosystem CI regression: `<ecosystem>/<suite>`

**Attribution:** Confirmed source

**Impact:** <one sentence describing the failing command, assertion, or blocked workflow>

#### Why this PR

<2-4 sentences connecting the PR change to the failure mechanism. State whether the PR caused a product regression or exposed a compatibility constraint, and name the actual fix owner when different.>

#### Suggested next step

<preferred fix and owner>. Verify with `<smallest useful command or focused test>`.

<details>
<summary>Evidence and attribution checks</summary>

- Current run: <run-url-or-id>
- Tested upstream: `<ecosystem>` at `<sha>`
- First bad / previous good: <pivot links or artifact evidence>
- Failure signature: <short-log-or-assertion-summary>
- Flaky/pre-existing check: <result>
- Config check: <feature enabled | not applicable>
- Alternatives ruled out: <surface pivot, downstream change, dependency update, or other candidates>

</details>
```

If the result is a correction rather than a blame comment, say so explicitly:

```md
<agent: daily-job rstack ecosystem-ci>

### Ecosystem CI attribution correction: `<ecosystem>/<suite>`

**Attribution:** This PR is not the source.

**Why:** <one-sentence reason the original surface attribution was misleading>

#### Actual source

<actual source or current inconclusive boundary, with a short mechanical explanation>

<details>
<summary>Evidence</summary>

- Surface pivot: <commit or PR>
- Contradicting evidence: <same-SHA result, older signature, disabled config, or unrelated diff>
- Actual-source evidence: <links or "insufficient evidence">

</details>
```
