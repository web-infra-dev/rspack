/** @type {import("@rspack/core").Configuration} */
module.exports = {
  resolve: {
    extensionAlias: {
      '.mjs': ['.mts'],
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'NormalModuleFactoryResolveErrorExtensionAliasTest',
          (nmf) => {
            nmf.hooks.resolveError.tap(
              'NormalModuleFactoryResolveErrorExtensionAliasTest',
              (resolveData, error) => {
                if (resolveData.request === './missing.mjs') {
                  expect(error.message).toContain('extension aliases');
                  resolveData.request = './extension-alias-fallback';
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
