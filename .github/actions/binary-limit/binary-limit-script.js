const fs = require('node:fs');
const { setTimeout: sleep } = require('node:timers/promises');

/**
 * @param {import("@octokit/rest")} github
 * @param {Number} limit
 */
module.exports = async function action({ github, context, limit }) {
  const headSize = fs.statSync(
    './crates/node_binding/rspack.linux-x64-gnu.node',
  ).size;
  console.log(`Head commit size: ${headSize}`);

  let baseCommit;
  let baseSize;
  try {
    ({ baseCommit, baseSize } = context.payload.pull_request
      ? await findBaseCommit(github, context)
      : await waitForBaseCommit(github, context));
  } catch (e) {
    if (e instanceof PendingBinaryDataError) {
      await tryComment(
        github,
        context,
        pendingBinarySizeComment(context, headSize, e),
      );
    }
    throw e;
  }

  console.log(`Base commit size: ${baseSize}`);

  await tryComment(
    github,
    context,
    compareBinarySize(headSize, baseSize, context, baseCommit),
  );

  const increasedSize = headSize - baseSize;
  if (increasedSize > limit) {
    throw new Error(
      `Binary size increased by ${increasedSize} bytes, exceeding the limit of ${limit} bytes`,
    );
  }
};

const PER_PAGE = 30;
const MAX_PAGES = 4;

// The parent's own CI publishes its size about four minutes in, so a commit merged
// right behind it can reach this job first. Observed window is one to two minutes;
// ten leaves room for a slow build.
const PENDING_POLL_INTERVAL_MS = 60_000;
const PENDING_POLL_ATTEMPTS = 10;

class PendingBinaryDataError extends Error {
  constructor(baseCommit, fallback) {
    super(
      `Base commit ${baseCommit.sha} triggered a linux binding build but its ` +
        'binary size data has not been generated yet. Please re-run this workflow ' +
        'once the ecosystem-benchmark run for that commit has published its data.',
    );
    this.baseCommit = baseCommit;
    this.fallback = fallback;
  }
}

// A trunk push has no PR to comment on, so an unpublished baseline can only surface
// as a red main — noise rather than a size problem. Wait for the parent's data instead
// of failing on it, and if it never lands, compare against the nearest ancestor that
// has data: that still measures real growth, it just spans more than one commit.
async function waitForBaseCommit(github, context) {
  let pending;
  for (let attempt = 1; ; attempt++) {
    try {
      return await findBaseCommit(github, context);
    } catch (e) {
      if (!(e instanceof PendingBinaryDataError)) throw e;
      pending = e;
    }

    if (attempt === PENDING_POLL_ATTEMPTS) break;
    console.log(
      `Base size data not published yet, retrying in ${PENDING_POLL_INTERVAL_MS / 1000}s (${attempt}/${PENDING_POLL_ATTEMPTS})`,
    );
    await sleep(PENDING_POLL_INTERVAL_MS);
  }

  if (!pending.fallback) throw pending;

  console.log(
    `Base data never arrived, comparing against ${pending.fallback.baseCommit.sha} instead`,
  );
  return pending.fallback;
}

// Baseline is the newest trunk commit already contained in the binding CI built,
// not the fork point: PR CI builds from the merge ref, so head size already
// includes that trunk tip. Walk trunk history skipping doc-only commits (they
// build no binding); the first build-triggering commit is decisive. Use its size
// data, or — when it isn't published yet (eco CI is slow) — fail loudly, attaching
// the nearest ancestor that already has data as a non-authoritative reference for
// a rough number.
async function findBaseCommit(github, context) {
  const { owner, repo } = context.repo;
  const baseSha = await resolveBaseSha(github, owner, repo, context);
  console.log(`Base trunk commit: ${baseSha}`);

  let pendingBase = null;

  for (let page = 1; page <= MAX_PAGES; page++) {
    const { data: commits } = await github.rest.repos.listCommits({
      owner,
      repo,
      sha: baseSha,
      per_page: PER_PAGE,
      page,
    });

    for (const commit of commits) {
      if (pendingBase) {
        const data = await fetchDataBySha(github, commit.sha);
        if (data?.size) {
          console.log(`Fallback reference ${commit.sha}: ${data.size}`);
          throw new PendingBinaryDataError(pendingBase, {
            baseCommit: commit,
            baseSize: data.size,
          });
        }
        continue;
      }

      if (!(await triggersBinaryBuild(github, owner, repo, commit.sha))) {
        console.log(`Commit ${commit.sha} is doc-only, skipping to parent`);
        continue;
      }

      const data = await fetchDataBySha(github, commit.sha);
      if (data?.size) {
        console.log(`Commit ${commit.sha} has binary size: ${data.size}`);
        return { baseCommit: commit, baseSize: data.size };
      }

      console.log(`Commit ${commit.sha} has no data yet, seeking a fallback`);
      pendingBase = commit;
    }

    if (commits.length < PER_PAGE) break;
  }

  if (pendingBase) {
    throw new PendingBinaryDataError(pendingBase, null);
  }

  throw new Error(
    `No base commit that triggered a linux binding build was found within ${MAX_PAGES} pages of commits from the base branch commit`,
  );
}

// Size data only exists for trunk commits, so the baseline must be a trunk commit
// — the newest one the binding under test actually contains.
async function resolveBaseSha(github, owner, repo, context) {
  const pr = context.payload.pull_request;
  const { data: commit } = await github.rest.repos.getCommit({
    owner,
    repo,
    ref: context.sha,
  });
  const [base, head] = commit.parents ?? [];

  // A trunk push measures the pushed commit itself, so its own parent is the baseline.
  if (!pr) {
    if (!base) {
      throw new Error(`Commit ${context.sha} has no parent to compare against`);
    }
    return base.sha;
  }

  if (commit.parents?.length !== 2 || head?.sha !== pr.head.sha) {
    console.log('context.sha is not a PR merge commit, using pr.base.sha');
    return pr.base.sha;
  }

  // For a standalone PR the base branch is the trunk, so the merged first parent
  // is the baseline outright.
  const stack = await resolveStack(github, owner, repo, pr);
  if (!stack) {
    return base.sha;
  }

  // In a stack the merge commits chain — each PR's merges into the one below —
  // so the first parent is another merge commit rather than a trunk commit. Take
  // the merge base with the trunk instead. Anchoring on the branch rather than on
  // the payload's `stack.base.sha` is deliberate: that sha is frozen at event time
  // while `refs/pull/N/merge` keeps being recomputed as the trunk advances, so it
  // can end up behind the trunk commit the binding contains and silently drag the
  // baseline backwards. The branch tip is always at or ahead of that commit, and
  // the merge base is the same however far ahead it is.
  console.log(`Stack trunk: ${stack.base.ref}`);
  const { data: comparison } =
    await github.rest.repos.compareCommitsWithBasehead({
      owner,
      repo,
      basehead: `${stack.base.ref}...${context.sha}`,
    });
  return comparison.merge_base_commit.sha;
}

// `stack` is missing from the payload when a PR's `opened` event outruns the stack
// being registered on GitHub — observed on #14907, created seconds before #14908,
// which did carry it. A re-run replays that same payload, so the check would stay
// red until the next push; read the PR back to settle it.
async function resolveStack(github, owner, repo, pr) {
  if (!pr.stack) {
    const { data } = await github.rest.pulls.get({
      owner,
      repo,
      pull_number: pr.number,
    });
    if (data.stack) {
      console.log(
        'stack was absent from the event payload, re-read from the API',
      );
      pr.stack = data.stack;
    }
  }
  return pr.stack;
}

// A binding is built (and size data produced) only for commits touching non-doc
// files, mirroring the `code_changed` filter in ci.yml that gates the binding build.
async function triggersBinaryBuild(github, owner, repo, sha) {
  const { data: commit } = await github.rest.repos.getCommit({
    owner,
    repo,
    ref: sha,
  });
  const files = commit.files ?? [];
  if (files.length === 0) return true;
  return files.some((file) => !isDocFile(file.filename));
}

function isDocFile(filename) {
  return (
    filename.endsWith('.md') ||
    filename.endsWith('.mdx') ||
    filename.startsWith('website/')
  );
}

async function tryComment(github, context, comment) {
  // A trunk push has no PR to comment on; it reports through the job status alone.
  if (!context.payload.pull_request) {
    console.log(comment);
    return;
  }
  try {
    await commentToPullRequest(github, context, comment);
  } catch (e) {
    console.error('Failed to comment on pull request:', e);
  }
}

async function commentToPullRequest(github, context, comment) {
  const { data: comments } = await github.rest.issues.listComments({
    owner: context.repo.owner,
    repo: context.repo.repo,
    issue_number: context.payload.number,
  });

  const prevComment = comments.filter(
    (comment) =>
      comment.user.login === 'github-actions[bot]' &&
      comment.body.startsWith(SIZE_LIMIT_HEADING),
  )[0];

  if (prevComment) {
    await github.rest.issues.updateComment({
      owner: context.repo.owner,
      repo: context.repo.repo,
      comment_id: prevComment.id,
      body: `${SIZE_LIMIT_HEADING}\n${comment}`,
    });
    return;
  }

  await github.rest.issues.createComment({
    owner: context.repo.owner,
    repo: context.repo.repo,
    issue_number: context.payload.number,
    body: `${SIZE_LIMIT_HEADING}\n${comment}`,
  });
}

// `ci-binary.json` is uploaded by the base commit's own CI right after the binding
// build; `rspack-build.json` carries the same measurement but only lands once the
// ecosystem benchmark completes, so it is a fallback for commits predating that job.
async function fetchDataBySha(github, sha) {
  const dir = `commits/${sha.slice(0, 2)}/${sha.slice(2)}`;
  return (
    (await fetchJson(github, `${dir}/ci-binary.json`)) ??
    (await fetchJson(github, `${dir}/rspack-build.json`))
  );
}

// Read via the authenticated Contents API rather than raw.githubusercontent.com:
// the CDN rate-limits anonymous requests per shared runner IP and 429s almost
// immediately, while the API uses the workflow token (5000 req/h) with octokit's
// built-in retry/throttling.
async function fetchJson(github, path) {
  console.log(
    'fetching',
    `${DATA_REPO.owner}/${DATA_REPO.repo}:${path}`,
    '...',
  );
  try {
    const { data } = await github.rest.repos.getContent({
      ...DATA_REPO,
      ref: DATA_REF,
      path,
    });
    return JSON.parse(Buffer.from(data.content, data.encoding).toString());
  } catch (e) {
    // 404 = data not published yet; other failures should surface their real cause.
    if (e.status === 404) return null;
    throw e;
  }
}

const SIZE_LIMIT_HEADING = '## 📦 Binary Size-limit';

const DATA_REPO = {
  owner: 'web-infra-dev',
  repo: 'rspack-ecosystem-benchmark',
};
const DATA_REF = 'data';

function runUrl(context) {
  return `${context.serverUrl}/${context.repo.owner}/${context.repo.repo}/actions/runs/${context.runId}`;
}

function comparingInfo(context, baseCommit) {
  const message = baseCommit.commit.message.split('\n')[0];
  const author = baseCommit.commit.author.name;
  const headSha = context.payload.pull_request?.head.sha || context.sha;
  return (
    `> Comparing [\`${headSha.slice(0, 7)}\`](${context.payload.repository.html_url}/commit/${headSha}) to  [${message} by ${author}](${baseCommit.html_url})\n\n` +
    stackNote(context.payload.pull_request)
  );
}

// Only the bottom PR of a stack sits directly on the trunk; for the ones above it
// the baseline is still the trunk, so the reported diff covers every PR below as
// well. Say so, otherwise the number reads as this PR's own contribution.
function stackNote(pr) {
  const trunk = pr?.stack?.base?.ref;
  if (!trunk || pr.base.ref === trunk) return '';
  return (
    '> [!NOTE]\n' +
    `> This PR is stacked on \`${pr.base.ref}\`. Sizes are compared against \`${trunk}\`, ` +
    'so the diff below covers the whole stack, not this PR alone.\n\n'
  );
}

function pendingBinarySizeComment(context, headSize, { baseCommit, fallback }) {
  let body =
    comparingInfo(context, baseCommit) +
    '⏳ The base commit triggered a linux binding build, but its binary size data ' +
    'has not been generated yet, so the size comparison is skipped.\n\n' +
    `Please [re-run this workflow](${runUrl(context)}) once the ecosystem-benchmark ` +
    'data for that commit is published.';

  if (fallback) {
    body += `\n\n${referenceComparison(headSize, fallback)}`;
  }

  return body;
}

function referenceComparison(headSize, { baseCommit, baseSize }) {
  const shortSha = baseCommit.sha.slice(0, 7);
  return (
    '> [!WARNING]\n' +
    "> **Reference only — not the real baseline.** The base commit's data isn't " +
    'ready yet, so this compares against the nearest earlier commit that has data ' +
    `([\`${shortSha}\`](${baseCommit.html_url})) for a rough estimate:\n` +
    '>\n' +
    `> ${sizeDiffLine(headSize, baseSize)}`
  );
}

function compareBinarySize(headSize, baseSize, context, baseCommit) {
  return comparingInfo(context, baseCommit) + sizeDiffLine(headSize, baseSize);
}

function sizeDiffLine(headSize, baseSize) {
  const diff = headSize - baseSize;
  const percentage = (Math.abs(diff / baseSize) * 100).toFixed(2);
  if (diff > 0) {
    return `❌ Size increased by ${toHumanReadable(diff)} from ${toHumanReadable(baseSize)} to ${toHumanReadable(headSize)} (⬆️${percentage}%)`;
  }
  if (diff < 0) {
    return `🎉 Size decreased by ${toHumanReadable(-diff)} from ${toHumanReadable(baseSize)} to ${toHumanReadable(headSize)} (⬇️${percentage}%)`;
  }
  return `🙈 Size remains the same at ${toHumanReadable(headSize)}`;
}

function toHumanReadable(size) {
  if (size < 1024) {
    return `${size}bytes`;
  }
  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(2)}KB`;
  }
  return `${(size / 1024 / 1024).toFixed(2)}MB`;
}
