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
  description: 'should invalidate only the loader rule whose options changed',
  async run() {
    const temp = fs.mkdtempSync(
      path.join(os.tmpdir(), 'rspack-loader-cache-rule-options-'),
    );
    const scriptRuns = path.join(temp, 'script-runs.txt');
    const styleRuns = path.join(temp, 'style-runs.txt');

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
                  loader: './css-loader.mjs',
                  options: { color, counterFile: styleRuns },
                  cache: true,
                },
              ],
            },
          ],
        },
      });
    }

    const first = createCompiler('first', 'red');
    const second = createCompiler('second', 'blue');
    try {
      expect((await runCompiler(first)).hasErrors()).toBe(false);
      expect(fs.readFileSync(scriptRuns, 'utf8')).toBe('x');
      expect(fs.readFileSync(styleRuns, 'utf8')).toBe('x');

      const stats = await runCompiler(second);
      expect(stats.hasErrors()).toBe(false);
      const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
      expect(cacheInfo?.['second|loader']).toEqual({
        hits: 1,
        misses: 1,
        reused: 1,
        stores: 1,
      });
      expect(fs.readFileSync(scriptRuns, 'utf8')).toBe('x');
      expect(fs.readFileSync(styleRuns, 'utf8')).toBe('xx');
      expect(
        fs.readFileSync(path.join(temp, 'second/style.js'), 'utf8'),
      ).toContain('color: blue');
    } finally {
      await Promise.all([closeCompiler(first), closeCompiler(second)]);
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
};
