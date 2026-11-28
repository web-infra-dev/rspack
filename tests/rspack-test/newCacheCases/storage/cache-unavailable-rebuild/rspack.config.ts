import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

const CACHE_LOGGER = 'rspack.cache';
const cacheLocation = path.join(import.meta.dirname, 'cache');
const warnings: string[] = [];
let compilerIndex = 0;
let current: Buffer;
let blocker: string;
let resolveUnavailable: () => void;
const unavailable = new Promise<void>((resolve) => {
  resolveUnavailable = resolve;
});

export default defineConfig({
  cache: {
    type: 'persistent',
    storage: { type: 'filesystem', location: cacheLocation },
  },
  infrastructureLogging: { level: 'none' },
  plugins: [
    definePlugin({
      name: 'UnavailableCacheTest',
      unavailable,
      apply(compiler) {
        const failCommit = compilerIndex++ === 0;
        let buildIndex = 0;
        compiler.hooks.infrastructureLog.tap(
          'UnavailableCacheTest',
          (name, type, args) => {
            if (name === CACHE_LOGGER && type === 'warn') {
              warnings.push(args[0]);
              if (
                args[0].includes(
                  'Filesystem cache unavailable for this session',
                )
              ) {
                resolveUnavailable();
              }
            }
          },
        );
        compiler.hooks.beforeRun.tap('UnavailableCacheTest', () => {
          if (failCommit && buildIndex === 1) {
            // Removing the fault must not re-enable persistence in the same session.
            fs.rmdirSync(blocker);
          }
        });
        compiler.hooks.afterCompile.tap('UnavailableCacheTest', () => {
          if (failCommit && buildIndex === 0) {
            current = fs.readFileSync(path.join(cacheLocation, 'CURRENT'));
            // A directory in place of CURRENT.next makes the first idle commit fail.
            blocker = path.join(cacheLocation, 'CURRENT.next');
            fs.mkdirSync(blocker);
          }
        });
        compiler.hooks.done.tap('UnavailableCacheTest', (stats) => {
          expect(stats.hasErrors()).toBe(false);
          expect(stats.hasWarnings()).toBe(false);
          buildIndex++;
        });
        // Cache shutdown hooks run after the native cache has flushed.
        compiler.cache.hooks.shutdown.tap('UnavailableCacheTest', () => {
          if (!failCommit) return;
          expect(buildIndex).toBe(3);
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
