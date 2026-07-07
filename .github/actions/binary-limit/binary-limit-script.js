const fs = require('node:fs');

/**
 * @param {import("@octokit/rest")} github
 * @param {Number} limit
 */
module.exports = async function action({ github, context, limit }) {
  let baseCommit;
  let baseSize;
  try {
    ({ baseCommit, baseSize } = await findBaseCommit(github, context));
  } catch (e) {
    // Data should exist for this base commit but hasn't been generated yet.
    // Still leave a comment (with a re-run link) before failing the job.
    if (e instanceof PendingBinaryDataError) {
      await commentBestEffort(
        github,
        context,
        pendingBinarySizeComment(context, e.baseCommit),
      );
    }
    throw e;
  }

  const headSize = fs.statSync(
    './crates/node_binding/rspack.linux-x64-gnu.node',
  ).size;

  console.log(`Base commit size: ${baseSize}`);
  console.log(`Head commit size: ${headSize}`);

  await commentBestEffort(
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

// Thrown when a base commit triggered a linux binding build but its binary size
// data has not been published yet, so a re-run is needed rather than a comparison.
class PendingBinaryDataError extends Error {
  constructor(baseCommit) {
    super(
      `Base commit ${baseCommit.sha} triggered a linux binding build but its ` +
        'binary size data has not been generated yet. Please re-run this workflow ' +
        'once the ecosystem-benchmark run for that commit has published its data.',
    );
    this.baseCommit = baseCommit;
  }
}

// The baseline is the base branch commit that the PR is merged onto at run time
// (`pr.base.sha`), not the fork point. For `pull_request` events CI builds the
// binding from the merge ref (PR head merged with the base tip), so the measured
// head size already includes the base branch's latest state and the correct
// baseline is that base commit. Walk main history from there toward the parent and:
//
//   - Skip doc-only commits: they don't trigger the ecosystem-benchmark build, so
//     they never have binary size data. The build (and thus data) is produced only
//     when a commit changes a non-doc file, mirroring that workflow's trigger
//     `paths-ignore: ['**/*.md', 'website/**']`.
//   - The first commit that DID trigger a build is decisive. If its size data is
//     published, use it. If not, the data simply hasn't been generated yet, so fail
//     loudly to force a re-run instead of silently comparing against an older
//     baseline that would misattribute intermediate changes to this PR.
async function findBaseCommit(github, context) {
  const { owner, repo } = context.repo;
  const pr = context.payload.pull_request;
  if (!pr) {
    throw new Error('binary-limit action requires pull_request context');
  }
  const baseSha = pr.base.sha;
  console.log(`Base branch commit: ${baseSha}`);

  for (let page = 1; page <= MAX_PAGES; page++) {
    const { data: commits } = await github.rest.repos.listCommits({
      owner,
      repo,
      sha: baseSha,
      per_page: PER_PAGE,
      page,
    });

    for (const commit of commits) {
      if (!(await triggersBinaryBuild(github, owner, repo, commit.sha))) {
        console.log(`Commit ${commit.sha} is doc-only, skipping to parent`);
        continue;
      }

      const data = await fetchDataBySha(commit.sha);
      if (data?.size) {
        console.log(`Commit ${commit.sha} has binary size: ${data.size}`);
        return { baseCommit: commit, baseSize: data.size };
      }

      throw new PendingBinaryDataError(commit);
    }

    if (commits.length < PER_PAGE) break;
  }

  throw new Error(
    `No base commit that triggered a linux binding build was found within ${MAX_PAGES} pages of commits from the base branch commit`,
  );
}

// A linux binding build (and thus binary size data) is produced only for a commit
// that changes at least one non-doc file, mirroring the ecosystem-benchmark
// workflow trigger `paths-ignore: ['**/*.md', 'website/**']`.
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

async function commentBestEffort(github, context, comment) {
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

async function fetchDataBySha(sha) {
  const dataUrl = `${DATA_URL_BASE}/commits/${sha.slice(0, 2)}/${sha.slice(2)}/rspack-build.json`;
  console.log('fetching', dataUrl, '...');
  const res = await fetch(dataUrl);
  // 404 means the size data hasn't been published for this commit yet; any other
  // failure is transient/unexpected and should surface its real cause instead of
  // being reported as "data not generated".
  if (res.status === 404) return null;
  if (!res.ok) {
    throw new Error(
      `Failed to fetch ${dataUrl}: ${res.status} ${res.statusText}`,
    );
  }
  return res.json();
}

const SIZE_LIMIT_HEADING = '## 📦 Binary Size-limit';

const DATA_URL_BASE =
  'https://raw.githubusercontent.com/web-infra-dev/rspack-ecosystem-benchmark/data';

function runUrl(context) {
  return `${context.serverUrl}/${context.repo.owner}/${context.repo.repo}/actions/runs/${context.runId}`;
}

function comparingInfo(context, baseCommit) {
  const message = baseCommit.commit.message.split('\n')[0];
  const author = baseCommit.commit.author.name;
  const headSha = context.payload.pull_request?.head.sha || context.sha;
  return `> Comparing [\`${headSha.slice(0, 7)}\`](${context.payload.repository.html_url}/commit/${headSha}) to  [${message} by ${author}](${baseCommit.html_url})\n\n`;
}

function pendingBinarySizeComment(context, baseCommit) {
  return (
    comparingInfo(context, baseCommit) +
    '⏳ The base commit triggered a linux binding build, but its binary size data ' +
    'has not been generated yet, so the size comparison is skipped.\n\n' +
    `Please [re-run this workflow](${runUrl(context)}) once the ecosystem-benchmark ` +
    'data for that commit is published.'
  );
}

function compareBinarySize(headSize, baseSize, context, baseCommit) {
  const info = comparingInfo(context, baseCommit);

  const diff = headSize - baseSize;
  const percentage = (Math.abs(diff / baseSize) * 100).toFixed(2);
  if (diff > 0) {
    return `${info}❌ Size increased by ${toHumanReadable(diff)} from ${toHumanReadable(baseSize)} to ${toHumanReadable(headSize)} (⬆️${percentage}%)`;
  }
  if (diff < 0) {
    return `${info}🎉 Size decreased by ${toHumanReadable(-diff)} from ${toHumanReadable(baseSize)} to ${toHumanReadable(headSize)} (⬇️${percentage}%)`;
  }
  return `${info}🙈 Size remains the same at ${toHumanReadable(headSize)}`;
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
