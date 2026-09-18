import path from 'node:path';

const versions = ['v1', 'v2', 'v1'];
let buildIndex = 0;

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, 'immutable')],
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.beforeCompile.tap('Test Plugin', function () {
          const version = versions[buildIndex++];
          if (version) {
            compiler.options.cache.version = version;
          }
        });
      },
    },
  ],
};
