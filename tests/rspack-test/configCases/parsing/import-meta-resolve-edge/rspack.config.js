'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  output: {
    assetModuleFilename: '[name][ext]',
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

module.exports.plugins = [
  (compiler) => {
    compiler.hooks.compilation.tap('CheckResolveArguments', (compilation) => {
      compilation.hooks.finishModules.tap(
        'CheckResolveArguments',
        (modules) => {
          expect(
            [...modules].some((module) => module.rawRequest === './request.js'),
          ).toBe(true);
        },
      );
    });
  },
];
