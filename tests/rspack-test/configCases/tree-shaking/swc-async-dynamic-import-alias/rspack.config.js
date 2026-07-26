const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    minimize: false,
  },
  resolve: {
    alias: {
      '@swc/helpers/_/_async_to_generator': path.resolve(
        __dirname,
        './fake-helper.js',
      ),
    },
  },
};
