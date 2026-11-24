import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'should track JS loader implementations and this.addBuildDependency() via build dependencies',
  async run() {
    const temp = fs.mkdtempSync(
      path.join(os.tmpdir(), 'rspack-loader-cache-build-deps-'),
    );

    async function runPersistentBuild({
      name,
      context,
      entry,
      directory,
      dist,
      rules,
      counterFile,
      expectedCounterLen,
      expectedCacheInfo,
      expectedBundleSubstrings = [],
    }) {
      const compiler = rspack({
        name,
        context,
        entry,
        mode: 'development',
        devtool: false,
        incremental: false,
        optimization: { minimize: false },
        output: { path: dist, filename: 'bundle.js' },
        cache: {
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
        module: { rules },
      });
      try {
        const stats = await runCompiler(compiler);
        expect(stats.hasErrors()).toBe(false);
        const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
        expect(cacheInfo?.[`${name}|loader`]).toEqual(expectedCacheInfo);
        expect(fs.readFileSync(counterFile, 'utf8')).toBe(
          'x'.repeat(expectedCounterLen),
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
      // 1. Local JS loader with transitive and cyclic imports, ignoring unrelated workspace package.json
      const localDir = path.join(temp, 'local');
      fs.mkdirSync(localDir, { recursive: true });
      const localCacheDir = path.join(localDir, 'cache');
      const localDistDir = path.join(localDir, 'dist');
      const localEntry = path.join(localDir, 'index.js');
      const localPkgJson = path.join(localDir, 'package.json');
      const localCounter = path.join(localDir, 'runs.txt');
      const helperJs = path.join(localDir, 'helper.mjs');
      const cycleA = path.join(localDir, 'cycle-a.mjs');
      const cycleB = path.join(localDir, 'cycle-b.mjs');
      const localLoader = path.join(localDir, 'local-loader.mjs');

      fs.writeFileSync(localEntry, "export default 'entry';\n");
      fs.writeFileSync(
        localPkgJson,
        JSON.stringify({ name: 'local-app', version: '1.0.0', type: 'module' }),
      );
      fs.writeFileSync(helperJs, "export const value = 'helper-v1';\n");
      fs.writeFileSync(cycleA, "import './cycle-b.mjs';\nexport const a = 1;\n");
      fs.writeFileSync(cycleB, "import './cycle-a.mjs';\nexport const b = 'cycle-v1';\n");
      fs.writeFileSync(
        localLoader,
        [
          "import fs from 'node:fs';",
          "import path from 'node:path';",
          "import './helper.mjs';",
          "import './cycle-a.mjs';",
          'export default function localLoader() {',
          '  const { counterFile, helperFile, cycleFile } = this.getOptions();',
          '  fs.mkdirSync(path.dirname(counterFile), { recursive: true });',
          "  fs.appendFileSync(counterFile, 'x');",
          '  return `export default ${JSON.stringify({',
          "    helper: fs.readFileSync(helperFile, 'utf8').trim(),",
          "    cycle: fs.readFileSync(cycleFile, 'utf8').trim(),",
          '  })};`;',
          '}',
        ].join('\n'),
      );

      const localRules = [
        {
          test: /index\.js$/,
          use: [
            {
              loader: localLoader,
              options: {
                counterFile: localCounter,
                helperFile: helperJs,
                cycleFile: cycleB,
              },
              cache: true,
            },
          ],
        },
      ];
      const missOne = { hits: 0, misses: 1, reused: 0, stores: 1 };
      const hitOne = { hits: 1, misses: 0, reused: 1, stores: 0 };

      await runPersistentBuild({
        name: 'local-1',
        context: localDir,
        entry: localEntry,
        directory: localCacheDir,
        dist: localDistDir,
        rules: localRules,
        counterFile: localCounter,
        expectedCounterLen: 1,
        expectedCacheInfo: missOne,
        expectedBundleSubstrings: ['helper-v1', 'cycle-v1'],
      });

      await runPersistentBuild({
        name: 'local-2-hit',
        context: localDir,
        entry: localEntry,
        directory: localCacheDir,
        dist: localDistDir,
        rules: localRules,
        counterFile: localCounter,
        expectedCounterLen: 1,
        expectedCacheInfo: hitOne,
        expectedBundleSubstrings: ['helper-v1', 'cycle-v1'],
      });

      // Editing an unrelated project package.json must not invalidate local loader caching.
      fs.writeFileSync(
        localPkgJson,
        JSON.stringify({ name: 'local-app', version: '2.0.0', type: 'module' }),
      );
      await runPersistentBuild({
        name: 'local-3-pkg-ignored',
        context: localDir,
        entry: localEntry,
        directory: localCacheDir,
        dist: localDistDir,
        rules: localRules,
        counterFile: localCounter,
        expectedCounterLen: 1,
        expectedCacheInfo: hitOne,
        expectedBundleSubstrings: ['helper-v1', 'cycle-v1'],
      });

      // Editing a statically imported helper file invalidates the loader etag.
      fs.writeFileSync(helperJs, "export const value = 'helper-v2';\n");
      await runPersistentBuild({
        name: 'local-4-helper-modified',
        context: localDir,
        entry: localEntry,
        directory: localCacheDir,
        dist: localDistDir,
        rules: localRules,
        counterFile: localCounter,
        expectedCounterLen: 2,
        expectedCacheInfo: missOne,
        expectedBundleSubstrings: ['helper-v2', 'cycle-v1'],
      });

      // Editing a transitively imported cyclic file invalidates the loader etag.
      fs.writeFileSync(cycleB, "import './cycle-a.mjs';\nexport const b = 'cycle-v2';\n");
      await runPersistentBuild({
        name: 'local-5-cycle-modified',
        context: localDir,
        entry: localEntry,
        directory: localCacheDir,
        dist: localDistDir,
        rules: localRules,
        counterFile: localCounter,
        expectedCounterLen: 3,
        expectedCacheInfo: missOne,
        expectedBundleSubstrings: ['helper-v2', 'cycle-v2'],
      });

      // 2. node_modules JS loader tracks both its entry file and enclosing package package.json
      const nmDir = path.join(temp, 'node-pkg');
      const pkgDir = path.join(nmDir, 'node_modules', 'pkg-loader');
      const pkgDistDir = path.join(pkgDir, 'dist');
      fs.mkdirSync(pkgDistDir, { recursive: true });
      const nmCacheDir = path.join(nmDir, 'cache');
      const nmDistDir = path.join(nmDir, 'dist');
      const nmEntry = path.join(nmDir, 'index.js');
      const nmCounter = path.join(nmDir, 'runs.txt');
      const nmPkgJson = path.join(pkgDir, 'package.json');
      const nmNestedPkgJson = path.join(pkgDistDir, 'package.json');
      const nmLoaderEntry = path.join(pkgDistDir, 'index.mjs');

      fs.writeFileSync(nmEntry, "export default 'nm';\n");
      fs.writeFileSync(
        nmPkgJson,
        JSON.stringify({ name: 'pkg-loader', version: '1.0.0', main: './dist/index.mjs' }),
      );
      fs.writeFileSync(nmNestedPkgJson, JSON.stringify({ type: 'module' }));
      fs.writeFileSync(
        nmLoaderEntry,
        [
          "import fs from 'node:fs';",
          "import path from 'node:path';",
          'export default function pkgLoader() {',
          '  const { counterFile, pkgFile } = this.getOptions();',
          '  fs.mkdirSync(path.dirname(counterFile), { recursive: true });',
          "  fs.appendFileSync(counterFile, 'x');",
          "  const version = JSON.parse(fs.readFileSync(pkgFile, 'utf8')).version;",
          '  return `export default ${JSON.stringify({ version, tag: "v1" })};`;',
          '}',
        ].join('\n'),
      );

      const nmRules = [
        {
          test: /index\.js$/,
          use: [
            {
              loader: 'pkg-loader',
              options: { counterFile: nmCounter, pkgFile: nmPkgJson },
              cache: true,
            },
          ],
        },
      ];

      await runPersistentBuild({
        name: 'nm-1',
        context: nmDir,
        entry: nmEntry,
        directory: nmCacheDir,
        dist: nmDistDir,
        rules: nmRules,
        counterFile: nmCounter,
        expectedCounterLen: 1,
        expectedCacheInfo: missOne,
        expectedBundleSubstrings: ['"version":"1.0.0"', '"tag":"v1"'],
      });

      await runPersistentBuild({
        name: 'nm-2-hit',
        context: nmDir,
        entry: nmEntry,
        directory: nmCacheDir,
        dist: nmDistDir,
        rules: nmRules,
        counterFile: nmCounter,
        expectedCounterLen: 1,
        expectedCacheInfo: hitOne,
        expectedBundleSubstrings: ['"version":"1.0.0"'],
      });

      // Updating package.json in node_modules invalidates the loader etag even when dist/index.mjs is unchanged.
      fs.writeFileSync(
        nmPkgJson,
        JSON.stringify({ name: 'pkg-loader', version: '1.0.1', main: './dist/index.mjs' }),
      );
      await runPersistentBuild({
        name: 'nm-3-pkg-updated',
        context: nmDir,
        entry: nmEntry,
        directory: nmCacheDir,
        dist: nmDistDir,
        rules: nmRules,
        counterFile: nmCounter,
        expectedCounterLen: 2,
        expectedCacheInfo: missOne,
        expectedBundleSubstrings: ['"version":"1.0.1"'],
      });

      // Updating the loader entry file in node_modules also invalidates the etag.
      fs.writeFileSync(
        nmLoaderEntry,
        fs.readFileSync(nmLoaderEntry, 'utf8').replace('"v1"', '"v2"'),
      );
      await runPersistentBuild({
        name: 'nm-4-entry-updated',
        context: nmDir,
        entry: nmEntry,
        directory: nmCacheDir,
        dist: nmDistDir,
        rules: nmRules,
        counterFile: nmCounter,
        expectedCounterLen: 3,
        expectedCacheInfo: missOne,
        expectedBundleSubstrings: ['"version":"1.0.1"'],
      });

      // 3. this.addBuildDependency() tracks directories, missing paths, and transitive JS imports across multiple modules
      const addBuildDir = path.join(temp, 'add-build-dep');
      fs.mkdirSync(addBuildDir, { recursive: true });
      const addBuildCacheDir = path.join(addBuildDir, 'cache');
      const addBuildDistDir = path.join(addBuildDir, 'dist');
      const addBuildEntry = path.join(addBuildDir, 'entry.js');
      const addBuildSecond = path.join(addBuildDir, 'second.js');
      const addBuildCounter = path.join(addBuildDir, 'runs.txt');
      const configJs = path.join(addBuildDir, 'config.mjs');
      const configTransitive = path.join(addBuildDir, 'config-helper.mjs');
      const trackedDir = path.join(addBuildDir, 'tracked-dir');
      const missingFile = path.join(addBuildDir, 'missing.txt');
      const fileDep = path.join(addBuildDir, 'file-dep.txt');
      const addBuildLoader = path.join(addBuildDir, 'add-build-loader.mjs');

      fs.mkdirSync(trackedDir, { recursive: true });
      fs.writeFileSync(addBuildEntry, "import second from './second.js';\nexport default second;\n");
      fs.writeFileSync(addBuildSecond, "export default 'second';\n");
      fs.writeFileSync(configJs, "import './config-helper.mjs';\n");
      fs.writeFileSync(configTransitive, "export const cfg = 'cfg-v1';\n");
      fs.writeFileSync(path.join(trackedDir, 'a.txt'), 'dir-a1');
      fs.writeFileSync(fileDep, 'file-v1');
      fs.writeFileSync(
        addBuildLoader,
        [
          "import fs from 'node:fs';",
          "import path from 'node:path';",
          'export default function addBuildLoader(source) {',
          '  const { counterFile, configJs, configTransitive, trackedDir, missingFile, fileDep } = this.getOptions();',
          '  fs.mkdirSync(path.dirname(counterFile), { recursive: true });',
          "  fs.appendFileSync(counterFile, 'x');",
          '  this.addBuildDependency(configJs);',
          '  this.addBuildDependency(trackedDir);',
          '  this.addBuildDependency(missingFile);',
          '  this.addDependency(fileDep);',
          "  const cfg = fs.readFileSync(configTransitive, 'utf8').trim();",
          '  const dirItems = fs.readdirSync(trackedDir).sort()',
          "    .map(name => `${name}:${fs.readFileSync(path.join(trackedDir, name), 'utf8')}`)",
          "    .join(',');",
          "  const missingVal = fs.existsSync(missingFile) ? fs.readFileSync(missingFile, 'utf8') : 'absent';",
          "  const fileVal = fs.readFileSync(fileDep, 'utf8');",
          '  if (this.resourcePath.endsWith("entry.js")) {',
          '    return `${source}\\nexport const meta = ${JSON.stringify({ cfg, dirItems, missingVal, fileVal })};`;',
          '  }',
          '  return `${source}\\nexport const tag = ${JSON.stringify({ cfg, dirItems, missingVal, fileVal })};`;',
          '}',
        ].join('\n'),
      );

      const addBuildRules = [
        {
          test: /\.(?:js)$/,
          use: [
            {
              loader: addBuildLoader,
              options: {
                counterFile: addBuildCounter,
                configJs,
                configTransitive,
                trackedDir,
                missingFile,
                fileDep,
              },
              cache: true,
            },
          ],
        },
      ];
      const missTwo = { hits: 0, misses: 2, reused: 0, stores: 2 };
      const hitTwo = { hits: 2, misses: 0, reused: 2, stores: 0 };
      const staleTwo = { hits: 2, misses: 0, reused: 0, stores: 2 };

      await runPersistentBuild({
        name: 'add-build-1',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 2,
        expectedCacheInfo: missTwo,
        expectedBundleSubstrings: ['cfg-v1', 'a.txt:dir-a1', 'absent', 'file-v1'],
      });

      await runPersistentBuild({
        name: 'add-build-2-hit',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 2,
        expectedCacheInfo: hitTwo,
        expectedBundleSubstrings: ['cfg-v1', 'a.txt:dir-a1', 'absent', 'file-v1'],
      });

      // Modifying a transitive JS dependency of this.addBuildDependency(configJs) invalidates both modules.
      fs.writeFileSync(configTransitive, "export const cfg = 'cfg-v2';\n");
      await runPersistentBuild({
        name: 'add-build-3-transitive-modified',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 4,
        expectedCacheInfo: staleTwo,
        expectedBundleSubstrings: ['cfg-v2'],
      });

      await runPersistentBuild({
        name: 'add-build-4-hit',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 4,
        expectedCacheInfo: hitTwo,
        expectedBundleSubstrings: ['cfg-v2'],
      });

      // Adding and modifying files inside a tracked directory invalidates via context_hash.
      fs.writeFileSync(path.join(trackedDir, 'b.txt'), 'dir-b1');
      await runPersistentBuild({
        name: 'add-build-5-dir-added',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 6,
        expectedCacheInfo: staleTwo,
        expectedBundleSubstrings: ['a.txt:dir-a1,b.txt:dir-b1'],
      });

      fs.writeFileSync(path.join(trackedDir, 'b.txt'), 'dir-b2');
      await runPersistentBuild({
        name: 'add-build-6-dir-modified',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 8,
        expectedCacheInfo: staleTwo,
        expectedBundleSubstrings: ['a.txt:dir-a1,b.txt:dir-b2'],
      });

      // Creating and deleting a previously missing addBuildDependency invalidates.
      fs.writeFileSync(missingFile, 'present');
      await runPersistentBuild({
        name: 'add-build-7-missing-created',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 10,
        expectedCacheInfo: staleTwo,
        expectedBundleSubstrings: ['"missingVal":"present"'],
      });

      fs.rmSync(missingFile);
      await runPersistentBuild({
        name: 'add-build-8-missing-deleted',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 12,
        expectedCacheInfo: staleTwo,
        expectedBundleSubstrings: ['"missingVal":"absent"'],
      });

      // Coexisting this.addDependency() file changes still invalidate cleanly.
      fs.writeFileSync(fileDep, 'file-v2');
      await runPersistentBuild({
        name: 'add-build-9-file-dep-modified',
        context: addBuildDir,
        entry: addBuildEntry,
        directory: addBuildCacheDir,
        dist: addBuildDistDir,
        rules: addBuildRules,
        counterFile: addBuildCounter,
        expectedCounterLen: 14,
        expectedCacheInfo: staleTwo,
        expectedBundleSubstrings: ['"fileVal":"file-v2"'],
      });
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
};
