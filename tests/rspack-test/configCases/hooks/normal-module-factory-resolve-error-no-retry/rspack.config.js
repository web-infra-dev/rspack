/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap(
          'NormalModuleFactoryResolveErrorNoRetryTest',
          (nmf) => {
            nmf.hooks.resolveError.tap(
              'NormalModuleFactoryResolveErrorNoRetryTest',
              (resolveData, error) => {
                expect(error.message).toContain('./missing-runtime');
                expect(resolveData.request).toBe('./missing-runtime');
                expect(resolveData.missingDependencies.length).toBeGreaterThan(
                  0,
                );
              },
            );
          },
        );
      },
    },
  ],
};
