import assert from 'node:assert/strict';
import { test } from 'node:test';
import { resolveBaseSha, stackNote } from './binary-limit-script.mjs';

const repoIdentity = { id: 1, owner: { login: 'web-infra-dev' } };

function parentPr(headRef, headSha, baseRef, baseSha, repo = repoIdentity) {
  return {
    head: { ref: headRef, sha: headSha, repo },
    base: { ref: baseRef, sha: baseSha, repo: repoIdentity },
  };
}

function fixture({
  baseRef = 'main',
  stack = null,
  parents = [{ sha: 'feature-base' }, { sha: 'feature-head' }],
  apiStack = null,
  mergeBase = 'trunk-common',
  parentPrs = [],
  baseRepo = repoIdentity,
} = {}) {
  const calls = [];
  const github = {
    rest: {
      repos: {
        async getCommit() {
          return { data: { parents } };
        },
        async compareCommitsWithBasehead(args) {
          calls.push(args.basehead);
          return {
            data: { merge_base_commit: mergeBase && { sha: mergeBase } },
          };
        },
      },
      pulls: {
        async get() {
          return { data: { stack: apiStack } };
        },
        async list(args) {
          calls.push(args.head);
          const branch = args.head.slice(args.head.indexOf(':') + 1);
          const matches = parentPrs.filter(
            (parent) => parent.head.ref === branch,
          );
          const start = (args.page - 1) * args.per_page;
          return { data: matches.slice(start, start + args.per_page) };
        },
      },
    },
  };
  const context = {
    sha: 'merge-ref',
    payload: {
      repository: { default_branch: 'main' },
      pull_request: {
        number: 123,
        base: { ref: baseRef, sha: 'event-base', repo: baseRepo },
        head: { sha: 'feature-head' },
        stack,
      },
    },
  };
  return { github, context, calls };
}

async function baseline(input) {
  return resolveBaseSha(input.github, 'web-infra-dev', 'rspack', input.context);
}

test('standalone PR uses the merge ref first parent', async () => {
  const input = fixture();
  assert.equal(await baseline(input), 'feature-base');
  assert.deepEqual(input.calls, []);
});

test('trunk push uses its parent without reading PR metadata', async () => {
  const input = fixture({ parents: [{ sha: 'trunk-parent' }] });
  delete input.context.payload.pull_request;
  assert.equal(await baseline(input), 'trunk-parent');
  assert.deepEqual(input.calls, []);
});

test('ordinary branch stack uses the common trunk commit, never its feature base', async () => {
  const input = fixture({
    baseRef: 'feat/portable-cache-1',
    parentPrs: [
      parentPr('feat/portable-cache-1', 'feature-base', 'main', 'trunk-tip'),
    ],
  });
  assert.equal(await baseline(input), 'trunk-common');
  assert.deepEqual(input.calls, [
    'web-infra-dev:feat/portable-cache-1',
    'main...merge-ref',
  ]);
  assert.equal(input.context.binaryLimitTrunk, 'main');
});

test('release-rooted branch stack uses v1.x rather than main', async () => {
  const input = fixture({
    baseRef: 'release-child',
    parentPrs: [
      parentPr('release-child', 'feature-base', 'release-root', 'root-tip'),
      parentPr('release-root', 'root-tip', 'v1.x', 'release-tip'),
    ],
  });
  assert.equal(await baseline(input), 'trunk-common');
  assert.deepEqual(input.calls, [
    'web-infra-dev:release-child',
    'web-infra-dev:release-root',
    'v1.x...merge-ref',
  ]);
  assert.equal(input.context.binaryLimitTrunk, 'v1.x');
});

test('direct release PR uses its published merge-ref parent', async () => {
  for (const baseRef of ['v1.x', 'release/2.3.0']) {
    const input = fixture({ baseRef });
    assert.equal(await baseline(input), 'feature-base');
    assert.deepEqual(input.calls, []);
  }
});

test('parent lookup paginates past same-named fork PRs', async () => {
  const fork = { id: 2, owner: { login: 'fork' } };
  const input = fixture({
    baseRef: 'feature-parent',
    parentPrs: [
      ...Array.from({ length: 100 }, () =>
        parentPr('feature-parent', 'feature-base', 'main', 'trunk-tip', fork),
      ),
      parentPr('feature-parent', 'feature-base', 'v1.x', 'release-tip'),
    ],
  });
  assert.equal(await baseline(input), 'trunk-common');
  assert.deepEqual(input.calls, [
    'web-infra-dev:feature-parent',
    'web-infra-dev:feature-parent',
    'v1.x...merge-ref',
  ]);
});

test('parent lookup fails when its bounded pages cannot prove uniqueness', async () => {
  const fork = { id: 2, owner: { login: 'fork' } };
  const input = fixture({
    baseRef: 'feature-parent',
    parentPrs: Array.from({ length: 400 }, () =>
      parentPr('feature-parent', 'feature-base', 'main', 'trunk-tip', fork),
    ),
  });
  await assert.rejects(baseline(input), /Parent PR search exceeded 4 pages/);
});

test('native stack keeps its declared trunk, including when fetched from the API', async () => {
  for (const source of ['payload', 'API']) {
    const nativeStack = { base: { ref: 'release' } };
    const input = fixture({
      baseRef: 'feature-parent',
      stack: source === 'payload' ? nativeStack : null,
      apiStack: source === 'API' ? nativeStack : null,
    });
    assert.equal(await baseline(input), 'trunk-common');
    assert.deepEqual(input.calls, ['release...merge-ref']);
  }
});

test('PR without a matching merge ref fails closed', async () => {
  for (const baseRef of ['main', 'feature-parent']) {
    for (const parents of [
      [],
      [{ sha: 'feature-base' }, { sha: 'old-head' }],
    ]) {
      const input = fixture({ baseRef, parents });
      await assert.rejects(baseline(input), /not the merge commit/);
      assert.deepEqual(input.calls, []);
    }
  }
});

test('unresolved, ambiguous, stale, cyclic, and fork parent links fail closed', async () => {
  await assert.rejects(
    baseline(fixture({ baseRef: 'feature-parent' })),
    /Cannot resolve parent PR/,
  );
  await assert.rejects(
    baseline(
      fixture({
        baseRef: 'feature-parent',
        parentPrs: [
          parentPr('feature-parent', 'feature-base', 'main', 'trunk-tip'),
          parentPr('feature-parent', 'feature-base', 'v1.x', 'release-tip'),
        ],
      }),
    ),
    /Ambiguous parent PR/,
  );
  await assert.rejects(
    baseline(
      fixture({
        baseRef: 'feature-parent',
        parentPrs: [
          parentPr('feature-parent', 'different-tip', 'main', 'trunk-tip'),
        ],
      }),
    ),
    /Cannot resolve parent PR/,
  );
  await assert.rejects(
    baseline(
      fixture({
        baseRef: 'feature-parent',
        parentPrs: [
          parentPr(
            'feature-parent',
            'feature-base',
            'feature-parent',
            'feature-base',
          ),
        ],
      }),
    ),
    /Cycle in parent PR branches/,
  );
  await assert.rejects(
    baseline(
      fixture({
        baseRef: 'feature-parent',
        parentPrs: [
          parentPr('feature-parent', 'feature-base', 'main', 'trunk-tip', {
            id: 2,
            owner: { login: 'fork' },
          }),
        ],
      }),
    ),
    /Cannot resolve parent PR/,
  );
});

test('missing base repository or merge base fails instead of selecting a feature commit', async () => {
  await assert.rejects(
    baseline(fixture({ baseRef: 'feature-parent', baseRepo: null })),
    /Cannot identify the base repository/,
  );
  await assert.rejects(
    baseline(
      fixture({
        baseRef: 'feature-parent',
        mergeBase: null,
        parentPrs: [
          parentPr('feature-parent', 'feature-base', 'main', 'trunk-tip'),
        ],
      }),
    ),
    /Cannot identify the common main ancestor/,
  );
});

test('comparison note covers ordinary stacks but not standalone PRs or pushes', () => {
  assert.match(
    stackNote({ base: { ref: 'feature-parent' } }, 'main'),
    /whole stack/,
  );
  assert.match(stackNote({ base: { ref: 'feature-parent' } }, 'v1.x'), /v1\.x/);
  assert.equal(stackNote({ base: { ref: 'main' } }, 'main'), '');
  assert.equal(stackNote(undefined, 'main'), '');
});
