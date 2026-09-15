const path = require('path');

const versions = ['v1', 'v2', 'v1'];
let buildIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(__dirname, './file.js')],
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
