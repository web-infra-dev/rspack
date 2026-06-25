const fs = require('node:fs');
const path = require('node:path');

const cacheDir = path.join(__dirname, 'node_modules/.cache/max-age');
// Change cache.version between restarts to create multiple persistent cache
// versions under the same storage directory.
const cacheVersions = ['v1', 'v2', 'v3'];
let buildIndex = 0;
let expiredGeneration;

const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const getCacheEntries = (directory) => {
  if (!fs.existsSync(directory)) {
    return [];
  }
  return fs
    .readdirSync(directory)
    .filter((name) => !name.startsWith('_') && !name.startsWith('.'))
    .sort();
};

const getCacheVersions = () => getCacheEntries(cacheDir);

// Persistent cache writes are queued in the background. Wait until the first
// version directory and `_meta` are both visible before starting the
// `maxAge` timeout, otherwise the timer could start before access time is
// recorded.
const waitForInitialGenerationWrite = async () => {
  for (let index = 0; index < 50; index++) {
    const versions = getCacheVersions();
    if (versions.length === 1 && fs.existsSync(path.join(cacheDir, '_meta'))) {
      return versions[0];
    }
    await wait(50);
  }

  throw new Error('Timed out waiting for the initial persistent cache version');
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
    maxAge: 1,
    storage: {
      type: 'filesystem',
      directory: cacheDir,
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeCompile.tap('Test Plugin', () => {
          if (buildIndex === 2) {
            const currentVersions = getCacheVersions();
            expect(currentVersions).toHaveLength(1);
            expect(currentVersions).not.toContain(expiredGeneration);
          }
          compiler.options.cache.version = cacheVersions[buildIndex];
        });
        compiler.hooks.done.tapPromise('Test Plugin', async () => {
          if (buildIndex === 0) {
            expiredGeneration = await waitForInitialGenerationWrite();
            // `maxAge` uses second-level timestamps and expires when
            // `lastAccess + maxAge < now`, so wait longer than one second.
            await wait(2200);
          }
          buildIndex++;
        });
      },
    },
  ],
};
