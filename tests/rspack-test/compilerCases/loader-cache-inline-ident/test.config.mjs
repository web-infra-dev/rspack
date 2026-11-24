import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { CssExtractRspackPlugin, rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    name: 'inline-ident',
    description:
      'should reuse loader cache for right-side loader invoked via importModule ??ident inline request',
    async run() {
      const temp = fs.mkdtempSync(
        path.join(os.tmpdir(), 'rspack-loader-cache-inline-ident-'),
      );
      const counterFile = path.join(temp, 'runs.txt');

      function createCompiler(name) {
        return rspack({
          name,
          context: import.meta.dirname,
          mode: 'development',
          devtool: false,
          entry: './index.js',
          output: { path: path.join(temp, name), filename: 'main.js' },
          cache: { type: 'memory' },
          experiments: { newCache: { loader: true, module: false } },
          module: {
            rules: [
              {
                test: /\.custom$/,
                type: 'javascript/auto',
                use: [
                  path.join(import.meta.dirname, 'pitch-import-loader.mjs'),
                  {
                    loader: path.join(import.meta.dirname, 'right-loader.mjs'),
                    options: { value: 'inline-cached', counterFile },
                    cache: true,
                    codeFilter: 'marker',
                  },
                ],
              },
            ],
          },
        });
      }

      const first = createCompiler('first');
      try {
        const firstStats = await runCompiler(first);
        expect(firstStats.hasErrors()).toBe(false);
        const { cacheInfo: firstCacheInfo } = firstStats.toJson({
          all: false,
          cacheInfo: true,
        });
        expect(firstCacheInfo?.['first|loader']).toEqual({
          hits: 0,
          misses: 1,
          reused: 0,
          stores: 1,
        });
        expect(fs.readFileSync(counterFile, 'utf8')).toBe('x');
        expect(
          fs.readFileSync(path.join(temp, 'first', 'main.js'), 'utf8'),
        ).toContain('inline-cached');

        const second = createCompiler('second');
        try {
          const secondStats = await runCompiler(second);
          expect(secondStats.hasErrors()).toBe(false);
          const { cacheInfo: secondCacheInfo } = secondStats.toJson({
            all: false,
            cacheInfo: true,
          });
          expect(secondCacheInfo?.['second|loader']).toEqual({
            hits: 1,
            misses: 0,
            reused: 1,
            stores: 0,
          });
          expect(fs.readFileSync(counterFile, 'utf8')).toBe('x');
          expect(
            fs.readFileSync(path.join(temp, 'second', 'main.js'), 'utf8'),
          ).toContain('inline-cached');
        } finally {
          await closeCompiler(second);
        }
      } finally {
        await closeCompiler(first);
        fs.rmSync(temp, { recursive: true, force: true });
      }
    },
  },
  {
    name: 'css-extract',
    description:
      'should reuse loader cache across CssExtractRspackPlugin inline importModule requests',
    async run() {
      const temp = fs.mkdtempSync(
        path.join(os.tmpdir(), 'rspack-loader-cache-css-extract-'),
      );
      const counterFile = path.join(temp, 'runs.txt');

      function createCompiler(name) {
        return rspack({
          name,
          context: import.meta.dirname,
          mode: 'development',
          devtool: false,
          entry: './css-entry.js',
          output: { path: path.join(temp, name), filename: 'main.js' },
          cache: { type: 'memory' },
          experiments: { newCache: { loader: true, module: false } },
          module: {
            rules: [
              {
                test: /\.css$/,
                type: 'javascript/auto',
                use: [
                  CssExtractRspackPlugin.loader,
                  { loader: 'css-loader', cache: true },
                  {
                    loader: path.join(
                      import.meta.dirname,
                      'count-css-loader.mjs',
                    ),
                    options: { counterFile },
                    cache: true,
                  },
                ],
              },
            ],
          },
          plugins: [new CssExtractRspackPlugin({ filename: 'main.css' })],
        });
      }

      const first = createCompiler('first');
      try {
        const firstStats = await runCompiler(first);
        expect(firstStats.hasErrors()).toBe(false);
        const { cacheInfo: firstCacheInfo } = firstStats.toJson({
          all: false,
          cacheInfo: true,
        });
        expect(firstCacheInfo?.['first|loader']).toEqual({
          hits: 0,
          misses: 1,
          reused: 0,
          stores: 1,
        });
        expect(fs.readFileSync(counterFile, 'utf8')).toBe('x');
        expect(
          fs.readFileSync(path.join(temp, 'first', 'main.css'), 'utf8'),
        ).toContain('crimson');

        const second = createCompiler('second');
        try {
          const secondStats = await runCompiler(second);
          expect(secondStats.hasErrors()).toBe(false);
          const { cacheInfo: secondCacheInfo } = secondStats.toJson({
            all: false,
            cacheInfo: true,
          });
          expect(secondCacheInfo?.['second|loader']).toEqual({
            hits: 1,
            misses: 0,
            reused: 1,
            stores: 0,
          });
          expect(fs.readFileSync(counterFile, 'utf8')).toBe('x');
          expect(
            fs.readFileSync(path.join(temp, 'second', 'main.css'), 'utf8'),
          ).toContain('crimson');
        } finally {
          await closeCompiler(second);
        }
      } finally {
        await closeCompiler(first);
        fs.rmSync(temp, { recursive: true, force: true });
      }
    },
  },
];
