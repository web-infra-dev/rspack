/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /module-[ab]\.js$/,
        use: './loader.js',
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('TestPlugin', (stats) => {
          const { errors, errorsCount, warnings, warningsCount } = stats.toJson(
            {
              all: false,
              errors: true,
              errorsCount: true,
              warnings: true,
              warningsCount: true,
            },
          );

          expect(errors).toHaveLength(200);
          expect(errorsCount).toBe(200);
          expect(warnings).toHaveLength(2);
          expect(warningsCount).toBe(2);

          const limitErrors = errors.filter(
            (error) => error.code === 'ModuleErrorsLimit',
          );
          expect(limitErrors).toHaveLength(2);
          expect(new Set(limitErrors.map((error) => error.moduleName))).toEqual(
            new Set(['./module-a.js', './module-b.js']),
          );
        });
      },
    },
  ],
};
