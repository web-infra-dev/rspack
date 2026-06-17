const fs = require('node:fs');
const path = require('node:path');

const cacheDir = path.join(__dirname, 'node_modules/.cache/max-generations');
// Change cache.version between restarts to create multiple persistent cache
// generations under the same compiler scope.
const cacheVersions = ['v1', 'v2', 'v3', 'v4'];
const seenGenerations = [];
let buildIndex = 0;

const getCacheEntries = (directory) => {
  if (!fs.existsSync(directory)) {
    return [];
  }
  return fs
    .readdirSync(directory)
    .filter((name) => !name.startsWith('_') && !name.startsWith('.'))
    .sort();
};

const getCompilerGenerations = () => {
  const compilerScopes = getCacheEntries(cacheDir);
  expect(compilerScopes.length).toBe(1);
  return getCacheEntries(path.join(cacheDir, compilerScopes[0]));
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
          if (buildIndex === 1) {
            seenGenerations[0] = getCompilerGenerations()[0];
          }
          if (buildIndex === 3) {
            const currentGenerations = getCompilerGenerations();
            expect(currentGenerations).toHaveLength(2);
            expect(currentGenerations).not.toContain(seenGenerations[0]);
          }
          compiler.options.cache.version = cacheVersions[buildIndex];
        });
        compiler.hooks.done.tap('Test Plugin', () => {
          buildIndex++;
        });
      },
    },
  ],
};
