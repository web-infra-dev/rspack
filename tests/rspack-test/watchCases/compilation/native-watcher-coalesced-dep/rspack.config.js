const path = require('node:path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      { test: /late\.js$/, use: path.resolve(__dirname, 'coalesce.loader.js') },
    ],
  },
};
