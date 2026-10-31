import fs from 'node:fs';
import path from 'node:path';
import { createFsFromVolume, Volume } from 'memfs';
import { defineConfig, definePlugin } from '@rspack/cli';

const CACHE_LOGGER = 'rspack.cache';
const cacheLocation = path.join(import.meta.dirname, 'cache');
const warnings: string[] = [];
let buildIndex = 0;
let current: Buffer;
let blocker: string;

export default defineConfig({
  entry: './a.js',
  cache: {
    type: 'persistent',
    storage: { type: 'filesystem', location: cacheLocation },
  },
  infrastructureLogging: { level: 'none' },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.outputFileSystem = createFsFromVolume(
          new Volume(),
        ) as unknown as typeof compiler.outputFileSystem;
        compiler.hooks.infrastructureLog.tap(
          'UnavailableCacheTest',
          (name, type, args) => {
            if (name === CACHE_LOGGER && type === 'warn') {
              warnings.push(args[0]);
            }
          },
        );
        compiler.hooks.beforeRun.tap('UnavailableCacheTest', () => {
          if (buildIndex === 1 && blocker) {
            // Removing the fault must not re-enable persistence in the same session.
            fs.rmdirSync(blocker);
          }
        });
        compiler.hooks.afterCompile.tap('UnavailableCacheTest', () => {
          if (buildIndex === 0) {
            current = fs.readFileSync(path.join(cacheLocation, 'CURRENT'));
            // A directory in place of CURRENT.next makes the first idle commit fail.
            blocker = path.join(cacheLocation, 'CURRENT.next');
            fs.mkdirSync(blocker);
          }
        });
        compiler.hooks.done.tap('UnavailableCacheTest', () => {
          buildIndex++;
          if (buildIndex !== 2) return;
          expect(warnings).toHaveLength(1);
          expect(warnings[0]).toContain('Committing CURRENT file failed');
          expect(fs.readFileSync(path.join(cacheLocation, 'CURRENT'))).toEqual(
            current,
          );
        });
      },
    }),
  ],
});
