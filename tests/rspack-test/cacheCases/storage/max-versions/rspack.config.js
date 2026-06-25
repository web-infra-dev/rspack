const fs = require('node:fs');
const path = require('node:path');

const cacheDir = path.join(__dirname, 'node_modules/.cache/max-versions');
// Change cache.version between restarts to create multiple persistent cache
// versions under the same storage directory.
const cacheVersions = ['v1', 'v2', 'v3', 'v4'];
let buildIndex = 0;
let firstVersion;

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

const waitForCacheVersions = async (predicate, errorMessage) => {
  for (let index = 0; index < 80; index++) {
    const versions = getCacheVersions();
    if (fs.existsSync(path.join(cacheDir, '_meta')) && predicate(versions)) {
      return versions;
    }
    await wait(50);
  }

  throw new Error(errorMessage);
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
    maxVersions: 2,
    storage: {
      type: 'filesystem',
      directory: cacheDir,
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeCompile.tap('Test Plugin', () => {
          compiler.options.cache.version = cacheVersions[buildIndex];
        });
        compiler.hooks.done.tapPromise('Test Plugin', async () => {
          if (buildIndex === 0) {
            [firstVersion] = await waitForCacheVersions(
              (versions) => versions.length === 1,
              'Timed out waiting for the first persistent cache version',
            );
            // maxVersions uses second-level access timestamps for LRU order.
            await wait(1200);
          }
          if (buildIndex === 1) {
            await waitForCacheVersions(
              (versions) => versions.length === 2,
              'Timed out waiting for the second persistent cache version',
            );
            await wait(1200);
          }
          if (buildIndex === 2) {
            const currentVersions = await waitForCacheVersions(
              (versions) =>
                versions.length === 2 && !versions.includes(firstVersion),
              'Timed out waiting for old persistent cache version cleanup',
            );
            expect(currentVersions).toHaveLength(2);
            expect(currentVersions).not.toContain(firstVersion);
          }
          buildIndex++;
        });
      },
    },
  ],
};
