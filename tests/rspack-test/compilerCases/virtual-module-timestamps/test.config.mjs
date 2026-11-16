import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { runNodeCase } from '@rspack/test-tools/helper/node-case';

const cases = ['development', 'none', 'production'].flatMap((mode) =>
  [false, true].map((dynamic) => ({
    name: `virtual-module-timestamps-${mode}-${dynamic ? 'dynamic' : 'static'}`,
    description: `preserves virtual timestamps across processes and invalidates changes in ${mode}`,
    async run() {
      const root = await fs.mkdtemp(
        path.join(os.tmpdir(), 'rspack-virtual-timestamps-'),
      );
      const initial = { en: 'hello', zh: '你好' };
      const rounds = [
        ['cold', 1, initial],
        ['warm', 1, initial],
        ['content', 2, { ...initial, en: 'howdy' }],
        ['content-warm', 2, { ...initial, en: 'howdy' }],
        ['revert', 1, initial],
        ['revert-warm', 1, initial],
        ['add', 1, { ...initial, fr: 'salut' }],
        ['add-warm', 1, { ...initial, fr: 'salut' }],
        ['remove', 1, { en: 'hello' }],
        ['remove-warm', 1, { en: 'hello' }],
        ['readonly-warm', 1, { en: 'hello' }, true],
        ['readonly-change', 3, { en: 'hello' }, true],
        ['readonly-original-warm', 1, { en: 'hello' }, true],
      ];
      try {
        for (const [round, value, translations, readonly = false] of rounds) {
          const result = await runNodeCase(
            new URL('./build.mjs', import.meta.url),
            [root, mode, value, translations, { readonly, dynamic }],
          );
          assert.deepEqual(
            result.output,
            {
              value,
              translations: Object.fromEntries(
                Object.entries(translations).map(([key, text]) => [
                  `./${key}.js`,
                  text,
                ]),
              ),
            },
            round,
          );
          if (round.endsWith('warm')) {
            assert.deepEqual(
              result.built,
              [],
              `${round}: ${JSON.stringify(result)}`,
            );
            assert.ok(result.restored.length > 0, round);
            assert.ok(Object.keys(result.cache).length > 0, round);
            for (const [namespace, counters] of Object.entries(result.cache)) {
              assert.equal(
                counters.reused,
                counters.hits + counters.misses,
                `${round}: ${namespace}`,
              );
            }
          } else if (round !== 'cold') {
            assert.ok(result.built.length > 0, round);
            if (round === 'add' || round === 'remove') {
              assert.ok(
                result.built.some((id) => id.includes('|sync|')),
                round,
              );
            }
          }
        }
      } finally {
        await fs.rm(root, { recursive: true, force: true });
      }
    },
  })),
);

export default [
  ...cases,
  {
    name: 'virtual-module-timestamps-path-conflicts',
    description:
      'rejects file and directory conflicts without changing virtual inputs',
    async run() {
      const root = await fs.mkdtemp(
        path.join(os.tmpdir(), 'rspack-virtual-paths-'),
      );
      try {
        const result = await runNodeCase(
          new URL('./build.mjs', import.meta.url),
          [
            root,
            'development',
            7,
            { en: 'hello' },
            { checkPathConflicts: true },
          ],
        );
        assert.deepEqual(result.output, {
          value: 7,
          translations: { './en.js': 'hello' },
        });
      } finally {
        await fs.rm(root, { recursive: true, force: true });
      }
    },
  },
  ...['child', 'build-dependency'].map((kind) => ({
    name: `virtual-module-timestamps-${kind}`,
    description: `restores virtual metadata before validating ${kind} snapshots`,
    async run() {
      const root = await fs.mkdtemp(
        path.join(os.tmpdir(), 'rspack-virtual-metadata-'),
      );
      try {
        const child = kind === 'child';
        for (const [index, version] of [1, 1, 2, 2, 1, 1].entries()) {
          const result = await runNodeCase(
            new URL('./build.mjs', import.meta.url),
            [
              root,
              'development',
              1,
              { en: 'hello' },
              {
                child,
                childValue: version + 100,
                buildDependency: child ? null : version,
              },
            ],
          );
          assert.equal(result.output.value, 1);
          if (child) assert.equal(result.childOutput.value, version + 100);
          if (index % 2 === 1) {
            assert.deepEqual(result.built, [], JSON.stringify(result));
            assert.deepEqual(result.childBuilt, [], JSON.stringify(result));
            if (child) assert.ok(result.childRestored.length > 0);
            for (const [namespace, counters] of Object.entries(result.cache)) {
              assert.equal(
                counters.reused,
                counters.hits + counters.misses,
                namespace,
              );
            }
          } else if (index > 0) {
            if (child) {
              assert.deepEqual(result.built, [], JSON.stringify(result));
              assert.ok(result.childBuilt.length > 0);
            } else {
              assert.ok(result.built.length > 0, JSON.stringify(result));
            }
          }
        }
      } finally {
        await fs.rm(root, { recursive: true, force: true });
      }
    },
  })),
  ...[false, true].flatMap((nativeWatcher) =>
    [false, true].map((persistent) => ({
      name: `virtual-module-timestamps-watch-${nativeWatcher}-${persistent}`,
      description:
        'invalidates live virtual changes and reversions without replaying cached timestamps',
      async run() {
        const root = await fs.mkdtemp(
          path.join(os.tmpdir(), 'rspack-virtual-watch-'),
        );
        try {
          await runNodeCase(new URL('./watch.mjs', import.meta.url), [
            root,
            nativeWatcher,
            persistent,
          ]);
        } finally {
          await fs.rm(root, { recursive: true, force: true });
        }
      },
    })),
  ),
];
