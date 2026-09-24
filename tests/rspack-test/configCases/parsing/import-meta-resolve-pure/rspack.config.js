'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    assetModuleFilename: '[path][name][ext]',
  },
  optimization: {
    minimize: false,
    usedExports: true,
    innerGraph: true,
  },
};

module.exports.module ??= {};
module.exports.module.parser ??= {};
module.exports.module.parser.javascript ??= {};
module.exports.module.parser.javascript.importMetaResolve = true;
