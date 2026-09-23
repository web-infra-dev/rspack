'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  output: {
    assetModuleFilename: '[name][ext]',
  },
  optimization: {
    minimize: false,
  },
  module: {
    parser: {
      javascript: {
        url: 'relative',
      },
    },
  },
};

module.exports.module ??= {};
module.exports.module.parser ??= {};
module.exports.module.parser.javascript ??= {};
module.exports.module.parser.javascript.importMetaResolve = true;
