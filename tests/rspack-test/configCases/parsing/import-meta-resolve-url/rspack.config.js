'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  resolve: { alias: { ignored: false } },
  output: {
    assetModuleFilename: '[name][ext]',
  },
  optimization: {
    minimize: false,
  },
};

module.exports.module ??= {};
module.exports.module.parser ??= {};
module.exports.module.parser.javascript ??= {};
module.exports.module.parser.javascript.importMetaResolve = true;
