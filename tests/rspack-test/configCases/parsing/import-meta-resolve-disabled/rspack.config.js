'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'node',
  output: {
    module: true,
    chunkFormat: 'module',
  },
  optimization: {
    minimize: false,
  },
  module: {
    parser: {
      javascript: {
        importMeta: {
          resolve: false,
        },
      },
    },
  },
};

module.exports.module ??= {};
module.exports.module.parser ??= {};
module.exports.module.parser.javascript ??= {};
module.exports.module.parser.javascript.importMetaResolve = true;
