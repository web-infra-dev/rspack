import assert from 'node:assert/strict';
import { test } from 'node:test';
import { resolveBaseSha, stackNote } from './binary-limit-script.mjs';

function fixture({
  baseRef = 'main',
  defaultBranch = 'main',
  stack = null,
  parents = [{ sha: 'feature-base' }, { sha: 'feature-head' }],
  apiStack = null,
  mergeBase = 'trunk-common',
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
          return { data: { merge_base_commit: mergeBase && { sha: mergeBase } } };
        },
      },
      pulls: {
        async get() {
          return { data: { stack: apiStack } };
        },
      },
    },
  };
  const context = {
    sha: 'merge-ref',
    payload: {
      repository: { default_branch: defaultBranch },
      pull_request: {
        number: 123,
        base: { ref: baseRef, sha: 'event-base' },
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
  const input = fixture({ baseRef: 'feat/portable-cache-1' });
  assert.equal(await baseline(input), 'trunk-common');
  assert.deepEqual(input.calls, ['main...merge-ref']);
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
    for (const parents of [[], [{ sha: 'feature-base' }, { sha: 'old-head' }]]) {
      const input = fixture({ baseRef, parents });
      await assert.rejects(baseline(input), /not the merge commit/);
      assert.deepEqual(input.calls, []);
    }
  }
});

test('missing trunk or merge base fails instead of selecting a feature commit', async () => {
  await assert.rejects(
    baseline(fixture({ baseRef: 'feature-parent', defaultBranch: null })),
    /Cannot identify the trunk branch/,
  );
  await assert.rejects(
    baseline(fixture({ baseRef: 'feature-parent', mergeBase: null })),
    /Cannot identify the common main ancestor/,
  );
});

test('comparison note covers ordinary stacks but not standalone PRs or pushes', () => {
  assert.match(stackNote({ base: { ref: 'feature-parent' } }, 'main'), /whole stack/);
  assert.equal(stackNote({ base: { ref: 'main' } }, 'main'), '');
  assert.equal(stackNote(undefined, 'main'), '');
});
