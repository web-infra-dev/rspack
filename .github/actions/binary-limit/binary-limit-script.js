const fs = require('node:fs');

/**
 * @param {import("@octokit/rest")} github
 * @param {Number} limit
 */
module.exports = async function action({ github, context, limit }) {
  const commits = await listCommitsFromMergeBase(github, context);

  let baseSize = 0;
  let baseCommit = null;

  for (const commit of commits) {
    console.log(commit.sha);
    try {
      const data = await fetchDataBySha(commit.sha);
      if (data?.size) {
        baseCommit = commit;
        baseSize = data.size;
        console.log(`Commit ${commit.sha} has binary size: ${data.size}`);
        break;
      }
    } catch (e) {
      console.log(e);
    }
  }

  if (!baseCommit) {
    const error = `No base binary size found within ${commits.length} commits`;
    console.log(error);
    throw new Error(error);
  }

  const headSize = fs.statSync(
    './crates/node_binding/rspack.linux-x64-gnu.node',
  ).size;

  console.log(`Base commit size: ${baseSize}`);
  console.log(`Head commit size: ${headSize}`);

  const comment = compareBinarySize(headSize, baseSize, context, baseCommit);

  try {
    await commentToPullRequest(github, context, comment);
  } catch (e) {
    console.error('Failed to comment on pull request:', e);
  }

  const increasedSize = headSize - baseSize;
  if (increasedSize > limit) {
    throw new Error(
      `Binary size increased by ${increasedSize} bytes, exceeding the limit of ${limit} bytes`,
    );
  }
};

// Start from the PR's merge base (fork point) rather than the base branch tip,
// so the size diff reflects only this PR's changes and not drift merged into
// the base branch after the PR forked. Walk its ancestors to the newest commit
// that has recorded binary size data.
async function listCommitsFromMergeBase(github, context) {
  const { owner, repo } = context.repo;
  const pr = context.payload.pull_request;

  const { data: comparison } =
    await github.rest.repos.compareCommitsWithBasehead({
      owner,
      repo,
      basehead: `${pr.base.sha}...${pr.head.sha}`,
    });
  const mergeBaseSha = comparison.merge_base_commit.sha;
  console.log(`Merge base commit: ${mergeBaseSha}`);

  const { data } = await github.rest.repos.listCommits({
    owner,
    repo,
    sha: mergeBaseSha,
    per_page: 30,
  });
  return data;
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

function fetchDataBySha(sha) {
  const dataUrl = `${DATA_URL_BASE}/commits/${sha.slice(0, 2)}/${sha.slice(2)}/rspack-build.json`;
  console.log('fetching', dataUrl, '...');
  return fetch(dataUrl).then((res) => res.json());
}

const SIZE_LIMIT_HEADING = '## 📦 Binary Size-limit';

const DATA_URL_BASE =
  'https://raw.githubusercontent.com/web-infra-dev/rspack-ecosystem-benchmark/data';

function compareBinarySize(headSize, baseSize, context, baseCommit) {
  const message = baseCommit.commit.message.split('\n')[0];
  const author = baseCommit.commit.author.name;
  const headSha = context.payload.pull_request?.head.sha || context.sha;

  const info = `> Comparing [\`${headSha.slice(0, 7)}\`](${context.payload.repository.html_url}/commit/${headSha}) to  [${message} by ${author}](${baseCommit.html_url})\n\n`;

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
