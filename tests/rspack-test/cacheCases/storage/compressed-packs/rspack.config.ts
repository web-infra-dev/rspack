import { defineConfig, definePlugin } from '@rspack/cli';
import type { Stats } from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

const cacheDir = path.join(import.meta.dirname, '.cache');
let closeIndex = 0;
let initialOutput: Buffer;
let initialMap: Buffer;

function packBytes(directory: string): number {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .reduce((sum, entry) => {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) return sum + packBytes(file);
      return sum + (entry.name.endsWith('.pack') ? fs.statSync(file).size : 0);
    }, 0);
}

export default defineConfig({
  mode: 'development',
  devtool: 'source-map',
  cache: {
    type: 'persistent',
    storage: { type: 'filesystem', directory: cacheDir },
  },
  module: {
    rules: [
      {
        test: /file\.js$/,
        use: [path.join(import.meta.dirname, 'loader.cjs')],
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        let stats: Stats;
        compiler.hooks.done.tap('CompressedPacksTest', (result) => {
          stats = result;
        });
        const close = compiler.close.bind(compiler);
        compiler.close = (callback) => {
          close((error) => {
            if (error) return callback(error);
            try {
              // Native close drains background storage writes before these assertions.
              const root = path.join(cacheDir, 'development');
              const directory = fs
                .readdirSync(root)
                .find((name) => name.startsWith('rspack_'))!;
              const bytes = packBytes(
                path.join(
                  root,
                  directory,
                  'occasion_source_map_dev_tool_plugin',
                ),
              );
              const output = Buffer.from(
                stats.compilation.getAsset('bundle.js')!.source.buffer(),
              );
              const map = Buffer.from(
                stats.compilation.getAsset('bundle.js.map')!.source.buffer(),
              );
              expect(map.length).toBeGreaterThan(512 * 1024);
              expect(bytes).toBeLessThan(map.length / 2);
              if (closeIndex === 0) {
                initialOutput = output;
                initialMap = map;
              } else if (closeIndex === 1) {
                expect(output).toEqual(initialOutput);
                expect(map).toEqual(initialMap);
                const logs = stats.toJson({
                  all: false,
                  logging: 'verbose',
                }).logging;
                const entry = logs?.[
                  'rspack.SourceMapDevToolPlugin'
                ]?.entries.find((entry) =>
                  entry.message?.startsWith('source map persistent cache:'),
                );
                expect(entry?.message).toContain('(1/1)');
              } else {
                expect(output.equals(initialOutput)).toBe(false);
                expect(map.equals(initialMap)).toBe(false);
                expect(map.toString()).toContain('export default 2');
              }
              closeIndex++;
              callback();
            } catch (error) {
              callback(error as Error);
            }
          });
        };
      },
    }),
  ],
});
