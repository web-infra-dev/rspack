const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  resolve: {
    alias: {
      m1: path.resolve(__dirname, 'does-not-exist.js'),
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'NormalModuleFactoryResolveErrorAliasTest',
          (nmf) => {
            nmf.hooks.resolveError.tap(
              'NormalModuleFactoryResolveErrorAliasTest',
              (resolveData, error) => {
                if (resolveData.request === 'm1') {
                  expect(error.message).toContain(
                    "Cannot find module 'm1' for matched aliased key 'm1'",
                  );
                  resolveData.request = './alias-fallback';
                  return { retry: true };
                }
              },
            );
          },
        );
      },
    },
  ],
};
