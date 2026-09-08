/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'NormalModuleFactoryResolveErrorFailedRetryTest',
          (nmf) => {
            let seen = 0;

            nmf.hooks.resolveError.tap(
              'NormalModuleFactoryResolveErrorFailedRetryTest',
              (resolveData) => {
                seen++;
                if (seen > 1) {
                  throw new Error(
                    'resolveError should not be called after retry',
                  );
                }
                resolveData.request = './still-missing-runtime';
                return { retry: true };
              },
            );
          },
        );
      },
    },
  ],
};
