/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'NormalModuleFactoryResolveErrorWatchTest',
          (nmf) => {
            nmf.hooks.resolveError.tap(
              'NormalModuleFactoryResolveErrorWatchTest',
              (resolveData) => {
                if (resolveData.request === './target') {
                  resolveData.request = './fallback';
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
