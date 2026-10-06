import { defineConfig, definePlugin } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';

const cacheDir = path.join(import.meta.dirname, '.cache');
let compilerIndex = 0;

function packFiles(directory: string): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) return packFiles(file);
    return entry.name.endsWith('.pack') ? [file] : [];
  });
}

export default defineConfig({
  mode: 'development',
  devtool: 'source-map',
  cache: {
    type: 'persistent',
    storage: {
      type: 'filesystem',
      directory: cacheDir,
      compression: 'lz4',
    },
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
        compiler.hooks.done.tap('CompressedPacksTest', (stats) => {
          const entries =
            stats.toJson({ all: false, logging: 'verbose' }).logging?.[
              'rspack.SourceMapDevToolPlugin'
            ]?.entries ?? [];
          const cacheEntry = entries.find(
            (entry) =>
              entry.type === 'cache' &&
              entry.message.startsWith('source map persistent cache:'),
          );
          const match = cacheEntry?.message.match(/\((\d+)\/(\d+)\)/);
          expect(match).toBeTruthy();
          expect(Number(match![1])).toBe(compilerIndex === 1 ? 1 : 0);
          expect(Number(match![2])).toBe(1);
          compilerIndex++;
        });
        const close = compiler.close.bind(compiler);
        compiler.close = (callback) => {
          close((error) => {
            if (error) return callback(error);
            try {
              // Native close drains background storage writes before inspecting packs.
              const packs = packFiles(cacheDir);
              const sourceMapPacks = packs.filter(
                (file) =>
                  file.includes(
                    `${path.sep}occasion_source_map_dev_tool_plugin${path.sep}`,
                  ) && fs.statSync(file).size > 1,
              );
              expect(sourceMapPacks.length).toBeGreaterThan(0);
              for (const file of sourceMapPacks) {
                const pack = fs.readFileSync(file);
                expect(pack[0]).toBe(0x01);
                // The decoded size requires later chunks to use a dictionary.
                expect(pack.readUInt32LE(1)).toBeGreaterThan(256 * 1024);
              }
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
