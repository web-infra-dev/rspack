const fs = require('node:fs');

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
    ({ baseCommit, baseSize } = await findBaseCommit(github, context));
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

// Baseline is the base commit actually merged into the PR to build the binding
// (the merge commit's first parent), not the fork point: PR CI builds from the
// merge ref, so head size already includes that base tip. Walk main history
// skipping doc-only commits (they build no binding); the first build-triggering
// commit is decisive. Use its size data, or — when it isn't published yet (eco CI
// is slow) — fail loudly, attaching the nearest ancestor that already has data as
// a non-authoritative reference for a rough number.
async function findBaseCommit(github, context) {
  const { owner, repo } = context.repo;
  const pr = context.payload.pull_request;
  if (!pr) {
    throw new Error('binary-limit action requires pull_request context');
  }
  const baseSha = await resolveBaseSha(github, owner, repo, context, pr);
  console.log(`Base branch commit: ${baseSha}`);

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

// For `pull_request` events `context.sha` is the ephemeral merge commit that CI
// checks out (`refs/pull/N/merge`); its first parent is the base commit actually
// merged in. `pr.base.sha` is only a stale snapshot of the base branch and drifts
// behind once main advances, so prefer the merge parent and fall back to it only
// when there is no merge commit (e.g. an unmergeable PR).
async function resolveBaseSha(github, owner, repo, context, pr) {
  const { data: mergeCommit } = await github.rest.repos.getCommit({
    owner,
    repo,
    ref: context.sha,
  });
  const [base, head] = mergeCommit.parents ?? [];
  if (mergeCommit.parents?.length === 2 && head?.sha === pr.head.sha) {
    return base.sha;
  }
  console.log('context.sha is not a PR merge commit, using pr.base.sha');
  return pr.base.sha;
}

// A binding is built (and size data produced) only for commits touching non-doc
// files, mirroring ecosystem-benchmark's `paths-ignore: ['**/*.md', 'website/**']`.
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
  return filename.endsWith('.md') || filename.startsWith('website/');
}

async function tryComment(github, context, comment) {
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

// Read via the authenticated Contents API rather than raw.githubusercontent.com:
// the CDN rate-limits anonymous requests per shared runner IP and 429s almost
// immediately, while the API uses the workflow token (5000 req/h) with octokit's
// built-in retry/throttling.
async function fetchDataBySha(github, sha) {
  const path = `commits/${sha.slice(0, 2)}/${sha.slice(2)}/rspack-build.json`;
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
  return `> Comparing [\`${headSha.slice(0, 7)}\`](${context.payload.repository.html_url}/commit/${headSha}) to  [${message} by ${author}](${baseCommit.html_url})\n\n`;
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
