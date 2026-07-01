/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'NormalModuleFactoryResolveErrorTest',
          (nmf) => {
            let seen = 0;

            nmf.hooks.resolveError.tapAsync(
              'NormalModuleFactoryResolveErrorTest',
              (resolveData, error, callback) => {
                expect(error.message).toContain('./missing-runtime');
                expect(resolveData.request).toBe('./missing-runtime');
                expect(resolveData.missingDependencies.length).toBeGreaterThan(
                  0,
                );

                seen++;
                resolveData.request = './runtime-notfound';
                callback(null, { retry: true });
              },
            );

            nmf.hooks.afterResolve.tap(
              'NormalModuleFactoryResolveErrorTest',
              (resolveData) => {
                if (resolveData.request.includes('runtime-notfound')) {
                  expect(seen).toBe(1);
                }
              },
            );
          },
        );
      },
    },
  ],
};
