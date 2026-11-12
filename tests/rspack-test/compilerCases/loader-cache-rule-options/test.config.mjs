import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';

const scriptFixture = path.join(
  import.meta.dirname,
  '../../multiCompilerCases/loader-cache',
);

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'should invalidate only the loader rule whose options or implementation changed',
  async run() {
    const temp = fs.mkdtempSync(
      path.join(os.tmpdir(), 'rspack-loader-cache-rule-options-'),
    );
    const scriptRuns = path.join(temp, 'script-runs.txt');
    const styleRuns = path.join(temp, 'style-runs.txt');
    const styleLoader = path.join(temp, 'css-loader.mjs');
    const originalStyleLoader = fs.readFileSync(
      path.join(import.meta.dirname, 'css-loader.mjs'),
      'utf8',
    );
    fs.writeFileSync(styleLoader, originalStyleLoader);

    function createCompiler(name, color) {
      return rspack({
        name,
        context: import.meta.dirname,
        mode: 'development',
        devtool: false,
        entry: {
          script: path.join(scriptFixture, 'index.js'),
          style: './style.css',
        },
        output: { path: path.join(temp, name) },
        cache: { type: 'memory' },
        experiments: { newCache: { loader: true, module: false } },
        module: {
          rules: [
            {
              test: /index\.js$/,
              use: [
                {
                  loader: path.join(scriptFixture, 'count-loader.mjs'),
                  options: { counterFile: scriptRuns },
                  cache: true,
                },
              ],
            },
            {
              test: /\.css$/,
              type: 'asset/source',
              use: [
                {
                  loader: styleLoader,
                  options: { color, counterFile: styleRuns },
                  cache: true,
                },
              ],
            },
          ],
        },
      });
    }

    async function check(name, color, styleRunCount) {
      const compiler = createCompiler(name, color);
      try {
        const stats = await runCompiler(compiler);
        expect(stats.hasErrors()).toBe(false);
        const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
        expect(cacheInfo?.[`${name}|loader`]).toEqual({
          hits: 1,
          misses: 1,
          reused: 1,
          stores: 1,
        });
        expect(fs.readFileSync(scriptRuns, 'utf8')).toBe('x');
        expect(fs.readFileSync(styleRuns, 'utf8')).toBe(
          'x'.repeat(styleRunCount),
        );
        expect(
          fs.readFileSync(path.join(temp, name, 'style.js'), 'utf8'),
        ).toContain(`color: ${color}`);
      } finally {
        await closeCompiler(compiler);
      }
    }

    const first = createCompiler('first', 'red');
    try {
      expect((await runCompiler(first)).hasErrors()).toBe(false);
      expect(fs.readFileSync(scriptRuns, 'utf8')).toBe('x');
      expect(fs.readFileSync(styleRuns, 'utf8')).toBe('x');

      // Changing and restoring options or loader contents replaces the etag
      // for the same cache key each time.
      await check('second', 'blue', 2);
      await check('third', 'red', 3);
      fs.appendFileSync(styleLoader, '\n// Change the loader file hash.\n');
      await check('updated-loader', 'red', 4);
      fs.writeFileSync(styleLoader, originalStyleLoader);
      await check('restored-loader', 'red', 5);
    } finally {
      await closeCompiler(first);
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
};
