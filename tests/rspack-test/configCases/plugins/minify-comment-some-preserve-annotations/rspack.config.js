const { rspack } = require('@rspack/core');
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
  entry: {
    a: './a',
    main: './index',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: true,
    // Keep the annotated expressions in this fixture. Export pruning is covered
    // independently by the CommonJS tree-shaking cases.
    usedExports: false,
  },
  plugins: [
    new rspack.SwcJsMinimizerRspackPlugin({
      minimizerOptions: {
        format: {
          comments: 'some',
          preserveAnnotations: true,
        },
      },
    }),
  ],
};
