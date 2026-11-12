import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'should invalidate the loader cache when the cache version changes',
  async run() {
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-loader-cache-'));
    const directory = path.join(temp, 'cache');
    const counterFile = path.join(temp, 'loader-runs.txt');
    const buildDependency = path.join(temp, 'config.js');
    const loader = path.join(
      import.meta.dirname,
      '../../multiCompilerCases/loader-cache/count-loader.mjs',
    );

    function createCompiler(mode, version, name) {
      return rspack({
        context: import.meta.dirname,
        entry: './index.js',
        mode,
        devtool: false,
        incremental: false,
        optimization: { minimize: false },
        output: { path: path.join(temp, 'dist') },
        cache: {
          type: 'persistent',
          name,
          version,
          buildDependencies: [buildDependency],
          maxMemoryGenerations: 0,
          storage: { type: 'filesystem', directory },
        },
        snapshot: { buildDependencies: { timestamp: false, hash: true } },
        experiments: {
          newCache: {
            codeGeneration: false,
            devtool: false,
            loader: true,
            minimize: false,
            module: true,
            resolver: false,
          },
        },
        module: {
          rules: [
            {
              test: /index\.js$/,
              use: [{ loader, options: { counterFile }, cache: true }],
            },
          ],
        },
      });
    }

    let loaderRuns = 0;
    // Closing a compiler must release its database without waiting for JS GC.
    const closedCompilers = [];
    async function checkBuild(compiler, expectedHits) {
      const stats = await runCompiler(compiler);
      expect(stats.hasErrors()).toBe(false);
      const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
      expect(cacheInfo?.['|loader']?.hits ?? 0).toBe(expectedHits);
      if (expectedHits === 0) loaderRuns++;
      expect(fs.readFileSync(counterFile, 'utf8')).toBe('x'.repeat(loaderRuns));
    }

    async function build(mode, version, expectedHits, name) {
      const compiler = createCompiler(mode, version, name);
      try {
        await checkBuild(compiler, expectedHits);
      } finally {
        await closeCompiler(compiler);
        closedCompilers.push(compiler);
      }
    }

    try {
      fs.writeFileSync(buildDependency, 'module.exports = 1;');
      await build('development', 'first', 0);

      fs.writeFileSync(buildDependency, 'module.exports = 42;');
      await build('development', 'first', 1);
      await build('production', 'first', 1);
      await build('production', 'second', 0);
      await build('production', 'second', 1, 'custom');

      const active = createCompiler('production', 'second', 'active');
      try {
        await checkBuild(active, 1);
        await build('production', 'third', 0, 'incompatible');
        await build('production', 'second', 1, 'compatible');
      } finally {
        await closeCompiler(active);
      }
      await build('production', 'third', 0);
      await build('production', 'third', 1, 'reopen');
      expect(fs.existsSync(path.join(directory, 'loader'))).toBe(true);
      expect(closedCompilers).toHaveLength(9);
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
};
