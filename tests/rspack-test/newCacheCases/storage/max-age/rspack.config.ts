import { defineConfig, definePlugin } from '@rspack/cli';
import type {
  FileSystemCacheOptions,
  PersistentCacheOptions,
} from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

const cacheDir = path.join(import.meta.dirname, 'node_modules/.cache/max-age');
const cacheVersions = ['v1', 'v2', 'v3'];
let buildIndex = 0;
let compilerCacheDirectory: string | undefined;

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const getCacheEntries = (directory: string) => {
  if (!fs.existsSync(directory)) {
    return [];
  }
  return fs
    .readdirSync(directory)
    .filter((name) => !name.startsWith('_') && !name.startsWith('.'))
    .sort();
};

const getCompilerCacheDirectories = () => getCacheEntries(cacheDir);

// Persistent cache writes are queued in the background. Wait until the first
// compiler cache directory and `_meta` are both visible before starting the
// `maxAge` timeout, otherwise the timer could start before access time is
// recorded.
const waitForInitialCacheWrite = async () => {
  for (let index = 0; index < 50; index++) {
    const directories = getCompilerCacheDirectories();
    if (
      directories.length === 1 &&
      fs.existsSync(path.join(cacheDir, '_meta'))
    ) {
      return directories[0];
    }
    await wait(50);
  }

  throw new Error('Timed out waiting for the initial compiler cache');
};

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    maxAge: 1,
    storage: {
      type: 'filesystem',
      location: cacheDir,
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const cache = compiler.options.cache as
          PersistentCacheOptions | FileSystemCacheOptions;
        compiler.hooks.beforeCompile.tap('Test Plugin', () => {
          if (buildIndex > 0) {
            const currentDirectories = getCompilerCacheDirectories();
            expect(currentDirectories).toEqual([compilerCacheDirectory]);
          }
          cache.version = cacheVersions[buildIndex];
        });
        compiler.hooks.done.tapPromise('Test Plugin', async () => {
          if (buildIndex === 0) {
            compilerCacheDirectory = await waitForInitialCacheWrite();
            // `maxAge` uses second-level timestamps and expires when
            // `lastAccess + maxAge < now`, so wait longer than one second.
            await wait(2200);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
