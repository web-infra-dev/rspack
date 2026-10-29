import fs from 'node:fs';
import path from 'node:path';
import { experiments } from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

const { RslibPlugin } = experiments;
const root = import.meta.dirname;
const counterFile = path.join(root, 'loader-runs.txt');
const dtsPath = path.join(root, 'dist/types/index.d.ts');

const warmLoaderRuns = 1;
let buildIndex = 0;
let readBytes = 0;
let firstOutput = '';

export default defineConfig({
  entry: './index.ts',
  mode: 'production',
  target: 'node',
  incremental: false,
  cache: { type: 'persistent' },
  experiments: { newCache: { loader: false, module: false } },
  output: {
    filename: 'output/main.js',
    library: { type: 'commonjs' },
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        type: 'javascript/auto',
        use: [
          {
            loader: 'builtin:swc-loader',
            cache: true,
            options: {
              jsc: {
                parser: { syntax: 'typescript' },
                experimental: { emitIsolatedDts: true },
              },
            },
          },
          {
            loader: path.join(root, 'count-loader.mjs'),
            cache: true,
            options: { counterFile },
          },
        ],
      },
    ],
  },
  plugins: [
    new RslibPlugin({
      emitDts: {
        rootDir: root,
        declarationDir: path.join(root, 'dist/types'),
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('IsolatedDtsTest', (stats) => {
          expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
          const runs = fs.readFileSync(counterFile, 'utf-8').length - readBytes;
          readBytes += runs;
          if (buildIndex === 0) {
            expect(runs).toBe(1);
            firstOutput = fs.readFileSync(dtsPath, 'utf-8');
            // Remove the declaration so the next compiler must emit it again,
            // even when SWC does not run.
            fs.rmSync(dtsPath);
          } else {
            expect(runs).toBe(warmLoaderRuns);
            expect(fs.readFileSync(dtsPath, 'utf-8')).toBe(firstOutput);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
