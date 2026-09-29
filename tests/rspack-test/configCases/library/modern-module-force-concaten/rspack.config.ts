import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { Compilation } from '@rspack/core';

export default defineConfig({
  entry: {
    a: './a.js',
    b: './b.cjs',
    c: './c.js',
    d: './d.mjs',
    e: './e/index.js',
    f: './f/index.js',
    g: './g/index.js',
    h: './h/file.png',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  externals: {
    path: 'node-commonjs path',
  },
  output: {
    module: true,
    filename: `[name].js`,
    library: { type: 'modern-module' },
    iife: false,
    chunkFormat: 'module',
  },
  optimization: {
    minimize: false,
  },
  plugins: [
    definePlugin(function () {
      const handler = (compilation: Compilation) => {
        compilation.hooks.afterProcessAssets.tap('testcase', (assets) => {
          expect(
            assets['a.js'].source().toString(),
            'ESM export should concat',
          ).toMatchFileSnapshotSync(
            path.join(import.meta.dirname, '__snapshot__', 'a.js.txt'),
          );
          expect(
            assets['b.js'].source().toString(),
            '.cjs should bail out',
          ).toMatchFileSnapshotSync(
            path.join(import.meta.dirname, '__snapshot__', 'b.js.txt'),
          );
          expect(
            assets['c.js'].source().toString(),
            'unambiguous should bail out',
          ).toMatchFileSnapshotSync(
            path.join(import.meta.dirname, '__snapshot__', 'c.js.txt'),
          );
          expect(
            assets['d.js'].source().toString(),
            '.mjs should concat',
          ).toMatchFileSnapshotSync(
            path.join(import.meta.dirname, '__snapshot__', 'd.js.txt'),
          );
          expect(
            assets['e.js'].source().toString(),
            '.cjs should bail out when bundling',
          ).toMatchFileSnapshotSync(
            path.join(import.meta.dirname, '__snapshot__', 'e.js.txt'),
          );
          expect(
            assets['f.js'].source().toString(),
            'CJS module with a direct external should bail out when bundling',
          ).toMatchFileSnapshotSync(
            path.join(import.meta.dirname, '__snapshot__', 'f.js.txt'),
          );
          expect(
            assets['g.js'].source().toString(),
            'harmony export should concat, even with bailout reason',
          ).toMatchFileSnapshotSync(
            path.join(import.meta.dirname, '__snapshot__', 'g.js.txt'),
          );
          expect(assets['h.js']).toBeUndefined();
          expect(
            Object.keys(assets).filter((name) => name.endsWith('.png')),
          ).toHaveLength(1);
        });
      };
      this.hooks.compilation.tap('testcase', handler);
    }),
  ],
});
