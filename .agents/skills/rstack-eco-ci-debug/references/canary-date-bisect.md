# Canary Date Bisect Tool

Use this tool when eco-ci history or release versions are too coarse to locate the Rspack PR or date that introduced or fixed a suite failure.

The goal is to test downstream code against specific `@rspack-canary/core` versions using `pnpm.overrides`, then binary-search by publish time or commit order.

## Preconditions

- Ask the user for:
  - The local Rspack checkout path.
  - The downstream project checkout path.
  - The narrowest failing command.
- Work in a clean downstream tree or record existing local changes first.
- Do not leave dependency overrides or lockfile changes behind unless the user asks.

## Find Candidate Canaries

Fetch available versions and publish times:

```bash
npm view @rspack-canary/core versions --json
npm view @rspack-canary/core time --json
```

Prefer canaries whose embedded short SHA can be resolved in the local Rspack checkout.

Map a canary short SHA to a full commit:

```bash
git -C <rspack-path> rev-parse <short-sha>
git -C <rspack-path> log -1 --pretty=format:'%h %s' <sha>
```

## Apply a Canary With pnpm.overrides

Prefer editing the downstream workspace root `package.json` and adding a temporary `pnpm.overrides` entry:

```json
{
  "pnpm": {
    "overrides": {
      "@rspack/core": "npm:@rspack-canary/core@<canary-version>"
    }
  }
}
```

If the project already centralizes overrides in another tracked package-manager config, use that location consistently and record the exact changed files before testing.

Install and verify:

```bash
pnpm -C <downstream-path> install
pnpm -C <downstream-path> why @rspack/core --depth 0
```

The test is invalid if `pnpm why` does not show the intended canary.

## Binary Search Loop

1. Pick a known-good canary and a known-bad canary.
2. Sort intermediate canaries by publish time, or by Rspack commit ancestry if publish time is misleading.
3. Test the midpoint canary.
4. Record:

```text
canary version | publish time | Rspack commit | result | signature
```

5. Continue until the first bad canary or first fixed canary is isolated.
6. If signatures differ, keep bisecting the signature change, not just pass/fail.

## Map the Date to a PR

Use the isolated commit to find the PR:

```bash
git -C <rspack-path> show <sha>
git -C <rspack-path> log --pretty=format:'%h %d %s' -n 20 <sha>
gh pr view <pr-number> --repo web-infra-dev/rspack
```

If only a PR head ref contains the commit:

```bash
git -C <rspack-path> ls-remote origin | rg '<short-sha>|<full-sha>'
```

Then inspect the PR diff and compare it to the failure signature before calling it the source.

## Restore Downstream State

After testing, remove the temporary override and reinstall if needed:

```bash
git -C <downstream-path> diff --name-only
git -C <downstream-path> restore -- <changed-files-from-the-diff-above>
pnpm -C <downstream-path> install
pnpm -C <downstream-path> why @rspack/core --depth 0
```

Only restore files that were actually changed by this workflow and are tracked by git. If files had pre-existing local changes, do not restore them blindly. Ask the user how to proceed.

## Output Format

```text
First bad canary: <version> (<publish-time>, <sha>)
Previous good canary: <version> (<publish-time>, <sha>)
Candidate PR: <pr-number> <title>
Failure signature: <short signature>
Confidence: high | medium | low
Reasoning: <why this is or is not enough to attribute>
```
