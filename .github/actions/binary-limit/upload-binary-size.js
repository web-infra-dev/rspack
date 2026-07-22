// Uploads the ci-profile binary size of a main/v1.x commit to the benchmark data
// branch, so PRs can read a baseline minutes after the binding build instead of
// waiting for the ecosystem benchmark (release build + 40min bench) to finish.
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const BINARY = 'crates/node_binding/rspack.linux-x64-gnu.node';
const DATA_DIR = '.benchmark-data';
const PUSH_RETRIES = 3;

const sha = process.env.GITHUB_SHA;
const token = process.env.PERF_DATA_TOKEN;
if (!sha || !token) {
  throw new Error('GITHUB_SHA and PERF_DATA_TOKEN are required');
}

const size = fs.statSync(BINARY).size;
console.log(`Binary size of ${sha}: ${size}`);

const remote = `https://x-access-token:${token}@github.com/web-infra-dev/rspack-ecosystem-benchmark.git`;
const relativePath = path.join('commits', sha.slice(0, 2), sha.slice(2));

function git(...args) {
  execFileSync('git', args, { cwd: DATA_DIR, stdio: 'inherit' });
}

// `git commit` exits non-zero with nothing staged, which a re-run of an already
// uploaded commit would hit.
function hasStagedChanges() {
  try {
    git('diff', '--cached', '--quiet');
    return false;
  } catch {
    return true;
  }
}

// Self-hosted runners reuse workspaces, so a leftover clone from a previous run
// would make `git clone` fail.
fs.rmSync(DATA_DIR, { recursive: true, force: true });
execFileSync(
  'git',
  [
    'clone',
    '--branch',
    'data',
    '--single-branch',
    '--depth',
    '1',
    remote,
    DATA_DIR,
  ],
  { stdio: 'inherit' },
);
git('config', 'user.name', 'github-actions[bot]');
git(
  'config',
  'user.email',
  '41898282+github-actions[bot]@users.noreply.github.com',
);

// The data branch is also written by the ecosystem benchmark upload, so a push can
// lose the race. Reset onto the new tip and re-apply rather than rebase: the file is
// commit-scoped, so re-writing it is always the correct resolution.
for (let attempt = 1; ; attempt++) {
  fs.mkdirSync(path.join(DATA_DIR, relativePath), { recursive: true });
  fs.writeFileSync(
    path.join(DATA_DIR, relativePath, 'ci-binary.json'),
    `${JSON.stringify({ size }, null, 2)}\n`,
  );
  git('add', path.join(relativePath, 'ci-binary.json'));
  if (!hasStagedChanges()) {
    console.log('Binary size already up to date, nothing to upload');
    break;
  }
  git('commit', '-m', `add ${sha.slice(0, 8)} ci binary size`);

  try {
    git('push');
    break;
  } catch (e) {
    if (attempt === PUSH_RETRIES) throw e;
    console.log(`Push rejected, retrying (${attempt}/${PUSH_RETRIES})`);
    git('fetch', '--depth', '1', 'origin', 'data');
    git('reset', '--hard', 'FETCH_HEAD');
  }
}
