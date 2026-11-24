import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';

const loaderPath = path.join(import.meta.dirname, 'loader.mjs');

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'should support loader cache object with identifier and buildDependencies',
  async run() {
    const temp = fs.mkdtempSync(
      path.join(os.tmpdir(), 'rspack-loader-cache-object-'),
    );
    const entry = path.join(temp, 'index.js');
    fs.copyFileSync(path.join(import.meta.dirname, 'index.js'), entry);

    const directory = path.join(temp, 'cache');
    const dist = path.join(temp, 'dist');
    const depFile = path.join(temp, 'dep.txt');
    const missingFile = path.join(temp, 'missing.txt');
    const depDir = path.join(temp, 'dep-dir');
    fs.mkdirSync(depDir, { recursive: true });
    fs.writeFileSync(depFile, 'dep-v1');
    fs.writeFileSync(path.join(depDir, 'a.txt'), 'a1');

    let totalRuns = 0;

    async function buildAndCheck({
      name,
      mode = 'development',
      cacheType = 'persistent',
      loaderCache,
      options,
      counterFile,
      expectedCacheInfo,
      expectRun,
      expectedBundleSubstrings = [],
    }) {
      const compiler = rspack({
        name,
        context: temp,
        entry,
        mode,
        devtool: false,
        incremental: false,
        optimization: { minimize: false },
        output: { path: dist, filename: 'bundle.js' },
        cache:
          cacheType === 'memory'
            ? { type: 'memory' }
            : {
                type: 'persistent',
                name,
                maxMemoryGenerations: 0,
                storage: { type: 'filesystem', directory },
              },
        experiments: {
          newCache: {
            codeGeneration: false,
            devtool: false,
            loader: true,
            minimize: false,
            module: false,
            resolver: false,
          },
        },
        module: {
          rules: [
            {
              test: /index\.js$/,
              use: [
                {
                  loader: loaderPath,
                  options,
                  cache: loaderCache,
                },
              ],
            },
          ],
        },
      });
      try {
        const stats = await runCompiler(compiler);
        expect(stats.hasErrors()).toBe(false);
        const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
        expect(cacheInfo?.[`${name ?? ''}|loader`]).toEqual(expectedCacheInfo);
        if (expectRun) totalRuns++;
        expect(fs.readFileSync(counterFile, 'utf8')).toBe(
          'x'.repeat(totalRuns),
        );
        const bundle = fs.readFileSync(path.join(dist, 'bundle.js'), 'utf8');
        for (const text of expectedBundleSubstrings) {
          expect(bundle).toContain(text);
        }
      } finally {
        await closeCompiler(compiler);
      }
    }

    try {
      const counterFile = path.join(temp, 'runs.txt');

      // 1. Empty cache object `{}` behaves like `true` (keep the first memory-cache
      //    compiler open so the shared memory loader cache stays alive).
      const memEntry = path.join(temp, 'mem-index.js');
      fs.copyFileSync(entry, memEntry);
      const createMemCompiler = (name) =>
        rspack({
          name,
          context: temp,
          entry: memEntry,
          mode: 'development',
          devtool: false,
          incremental: false,
          optimization: { minimize: false },
          output: { path: path.join(temp, 'mem-dist'), filename: 'bundle.js' },
          cache: { type: 'memory' },
          experiments: { newCache: { loader: true, module: false } },
          module: {
            rules: [
              {
                test: /mem-index\.js$/,
                use: [
                  {
                    loader: loaderPath,
                    options: { counterFile },
                    cache: {},
                  },
                ],
              },
            ],
          },
        });
      const memFirst = createMemCompiler('empty-object-1');
      try {
        const firstStats = await runCompiler(memFirst);
        expect(firstStats.hasErrors()).toBe(false);
        const firstInfo = firstStats.toJson({ all: false, cacheInfo: true });
        expect(firstInfo.cacheInfo?.['empty-object-1|loader']).toEqual({
          hits: 0,
          misses: 1,
          reused: 0,
          stores: 1,
        });

        const memSecond = createMemCompiler('empty-object-2');
        try {
          const secondStats = await runCompiler(memSecond);
          expect(secondStats.hasErrors()).toBe(false);
          const secondInfo = secondStats.toJson({ all: false, cacheInfo: true });
          expect(secondInfo.cacheInfo?.['empty-object-2|loader']).toEqual({
            hits: 1,
            misses: 0,
            reused: 1,
            stores: 0,
          });
        } finally {
          await closeCompiler(memSecond);
        }
      } finally {
        await closeCompiler(memFirst);
      }
      totalRuns += 1;
      expect(fs.readFileSync(counterFile, 'utf8')).toBe('x');

      // 2. `cache.identifier` + `cache.buildDependencies` (relative & absolute,
      //    deduplication, order independence, missing file, directory, persistent reopen).
      const miss = { hits: 0, misses: 1, reused: 0, stores: 1 };
      const hit = { hits: 1, misses: 0, reused: 1, stores: 0 };
      const baseOptions = { counterFile, depFile, depDir };
      const createCacheOption = (mode, buildDependencies) => ({
        identifier: JSON.stringify({ mode }),
        buildDependencies,
      });

      await buildAndCheck({
        name: 'id-dev-1',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: [
          '"mode":"development"',
          '"dep":"dep-v1"',
          '"dir":"a.txt:a1"',
        ],
      });

      // Reordered and duplicated relative/absolute paths still hit persistent cache after reopen.
      await buildAndCheck({
        name: 'id-dev-2',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          depDir,
          depFile,
          './missing.txt',
          './dep.txt',
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: hit,
        expectRun: false,
        expectedBundleSubstrings: [
          '"mode":"development"',
          '"dep":"dep-v1"',
          '"dir":"a.txt:a1"',
        ],
      });

      // Changing identifier invalidates etag and refreshes output.
      await buildAndCheck({
        name: 'id-prod-1',
        mode: 'production',
        loaderCache: createCacheOption('production', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['"mode":"production"'],
      });

      await buildAndCheck({
        name: 'id-prod-2',
        mode: 'production',
        loaderCache: createCacheOption('production', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: hit,
        expectRun: false,
        expectedBundleSubstrings: ['"mode":"production"'],
      });

      // Switching back to previous identifier misses because the single key's etag changed.
      await buildAndCheck({
        name: 'id-dev-3',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['"mode":"development"'],
      });

      // Modifying a tracked file invalidates etag.
      fs.writeFileSync(depFile, 'dep-v2');
      await buildAndCheck({
        name: 'dep-modified',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['"dep":"dep-v2"'],
      });

      // Creating a previously missing dependency invalidates etag.
      fs.writeFileSync(missingFile, 'created');
      await buildAndCheck({
        name: 'missing-created',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
      });

      // Deleting that dependency again invalidates etag.
      fs.rmSync(missingFile);
      await buildAndCheck({
        name: 'missing-deleted',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
      });

      // Adding a file inside a tracked directory invalidates etag.
      fs.writeFileSync(path.join(depDir, 'b.txt'), 'b1');
      await buildAndCheck({
        name: 'dir-added',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['"dir":"a.txt:a1,b.txt:b1"'],
      });

      // Modifying a file inside a tracked directory invalidates etag.
      fs.writeFileSync(path.join(depDir, 'b.txt'), 'b2');
      await buildAndCheck({
        name: 'dir-modified',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['"dir":"a.txt:a1,b.txt:b2"'],
      });

      // Unchanged state hits cache across compiler reopen.
      await buildAndCheck({
        name: 'dir-hit',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './dep.txt',
          './missing.txt',
          depDir,
        ]),
        options: baseOptions,
        counterFile,
        expectedCacheInfo: hit,
        expectRun: false,
        expectedBundleSubstrings: ['"dir":"a.txt:a1,b.txt:b2"'],
      });

      // 2b. Transitive and cyclic JS build dependencies resolve recursively like top-level `cache.buildDependencies`.
      const configJs = path.join(temp, 'config.mjs');
      const helperJs = path.join(temp, 'dep.mjs');
      const cycleA = path.join(temp, 'cycle-a.mjs');
      const cycleB = path.join(temp, 'cycle-b.mjs');
      fs.writeFileSync(configJs, "import './dep.mjs';\n");
      fs.writeFileSync(helperJs, "export default 'helper-v1';\n");
      fs.writeFileSync(cycleA, "import './cycle-b.mjs';\nexport const a = 1;\n");
      fs.writeFileSync(cycleB, "import './cycle-a.mjs';\nexport const b = 'cycle-v1';\n");
      const transitiveOptions = {
        counterFile,
        depFile: helperJs,
        extraFiles: [cycleB],
      };

      await buildAndCheck({
        name: 'transitive-1',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './config.mjs',
          './cycle-a.mjs',
        ]),
        options: transitiveOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['helper-v1', 'cycle-v1'],
      });

      await buildAndCheck({
        name: 'transitive-hit',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './config.mjs',
          './cycle-a.mjs',
        ]),
        options: transitiveOptions,
        counterFile,
        expectedCacheInfo: hit,
        expectRun: false,
        expectedBundleSubstrings: ['helper-v1', 'cycle-v1'],
      });

      fs.writeFileSync(helperJs, "export default 'helper-v2';\n");
      await buildAndCheck({
        name: 'transitive-helper-modified',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './config.mjs',
          './cycle-a.mjs',
        ]),
        options: transitiveOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['helper-v2', 'cycle-v1'],
      });

      await buildAndCheck({
        name: 'transitive-helper-hit',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './config.mjs',
          './cycle-a.mjs',
        ]),
        options: transitiveOptions,
        counterFile,
        expectedCacheInfo: hit,
        expectRun: false,
        expectedBundleSubstrings: ['helper-v2', 'cycle-v1'],
      });

      fs.writeFileSync(cycleB, "import './cycle-a.mjs';\nexport const b = 'cycle-v2';\n");
      await buildAndCheck({
        name: 'transitive-cycle-modified',
        mode: 'development',
        loaderCache: createCacheOption('development', [
          './config.mjs',
          './cycle-a.mjs',
        ]),
        options: transitiveOptions,
        counterFile,
        expectedCacheInfo: miss,
        expectRun: true,
        expectedBundleSubstrings: ['helper-v2', 'cycle-v2'],
      });

      // 3. Non-JSON-serializable options still disable caching safely even with `cache` object.
      for (const name of ['non-serializable-1', 'non-serializable-2']) {
        await buildAndCheck({
          name,
          mode: 'development',
          loaderCache: { identifier: 'v1' },
          options: { ...baseOptions, nonSerializable: () => {} },
          counterFile,
          expectedCacheInfo: undefined,
          expectRun: true,
        });
      }

      // 4. Invalid `cache` config values throw validation errors.
      const invalidConfigs = [
        [
          'invalid',
          'Expected a boolean or object at `ruleSet[1].rules[0].use[0].cache`.',
        ],
        [
          null,
          'Expected a boolean or object at `ruleSet[1].rules[0].use[0].cache`.',
        ],
        [
          [],
          'Expected a boolean or object at `ruleSet[1].rules[0].use[0].cache`.',
        ],
        [
          { identifier: 123 },
          'Expected a string at `ruleSet[1].rules[0].use[0].cache.identifier`.',
        ],
        [
          { buildDependencies: 'a' },
          'Expected a string array at `ruleSet[1].rules[0].use[0].cache.buildDependencies`.',
        ],
        [
          { buildDependencies: [123] },
          'Expected a string array at `ruleSet[1].rules[0].use[0].cache.buildDependencies`.',
        ],
      ];
      for (const [invalidCache, expectedMessage] of invalidConfigs) {
        const compiler = rspack({
          context: temp,
          entry,
          module: {
            rules: [
              {
                test: /index\.js$/,
                use: [{ loader: loaderPath, cache: invalidCache }],
              },
            ],
          },
        });
        try {
          await expect(runCompiler(compiler)).rejects.toThrow(expectedMessage);
        } finally {
          await closeCompiler(compiler);
        }
      }
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
};
