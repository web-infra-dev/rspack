const fs = require('node:fs');
const path = require('node:path');

const cacheDir = path.join(__dirname, 'node_modules/.cache/max-generations');
// Change cache.version between restarts to create multiple persistent cache
// generations under the same storage directory.
const cacheVersions = ['v1', 'v2', 'v3', 'v4'];
let buildIndex = 0;
let firstGeneration;

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

const getCacheGenerations = () => getCacheEntries(cacheDir);

const waitForCacheGenerations = async (predicate, errorMessage) => {
  for (let index = 0; index < 80; index++) {
    const generations = getCacheGenerations();
    if (fs.existsSync(path.join(cacheDir, '_meta')) && predicate(generations)) {
      return generations;
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
    storage: {
      type: 'filesystem',
      directory: cacheDir,
      maxGenerations: 2,
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
            [firstGeneration] = await waitForCacheGenerations(
              (generations) => generations.length === 1,
              'Timed out waiting for the first persistent cache generation',
            );
            // maxGenerations uses second-level access timestamps for LRU order.
            await wait(1200);
          }
          if (buildIndex === 1) {
            await waitForCacheGenerations(
              (generations) => generations.length === 2,
              'Timed out waiting for the second persistent cache generation',
            );
            await wait(1200);
          }
          if (buildIndex === 2) {
            const currentGenerations = await waitForCacheGenerations(
              (generations) =>
                generations.length === 2 &&
                !generations.includes(firstGeneration),
              'Timed out waiting for old persistent cache generation cleanup',
            );
            expect(currentGenerations).toHaveLength(2);
            expect(currentGenerations).not.toContain(firstGeneration);
          }
          buildIndex++;
        });
      },
    },
  ],
};
