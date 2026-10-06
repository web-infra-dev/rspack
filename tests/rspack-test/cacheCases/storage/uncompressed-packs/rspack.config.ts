import { defineConfig, definePlugin } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';

const cacheDir = path.join(import.meta.dirname, '.cache');

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
      compression: false,
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
        const close = compiler.close.bind(compiler);
        compiler.close = (callback) => {
          close((error) => {
            if (error) return callback(error);
            try {
              // Native close drains background storage writes before inspecting packs.
              const packs = packFiles(cacheDir);
              const sourceMapPacks = packs.filter((file) =>
                file.includes(
                  `${path.sep}occasion_source_map_dev_tool_plugin${path.sep}`,
                ),
              );
              expect(sourceMapPacks.length).toBeGreaterThan(0);
              for (const file of packs) {
                expect(fs.readFileSync(file)[0]).toBe(0x00);
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
