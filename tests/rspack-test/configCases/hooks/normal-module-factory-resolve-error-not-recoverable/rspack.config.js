/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'NormalModuleFactoryResolveErrorNotRecoverableTest',
          (nmf) => {
            nmf.hooks.resolveError.tap(
              'NormalModuleFactoryResolveErrorNotRecoverableTest',
              () => {
                throw new Error(
                  'resolveError should not be called for non-module-not-found resolver errors',
                );
              },
            );
          },
        );
      },
    },
  ],
};
