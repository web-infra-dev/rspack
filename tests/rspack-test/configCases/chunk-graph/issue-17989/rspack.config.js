/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    a: './entry-a',
    b: './entry-b',
  },
  optimization: {
    sideEffects: true,
    providedExports: true,
    usedExports: true,
    concatenateModules: false,
    moduleIds: 'named',
    // avoid inlineExports to change to chunk graph structure
    inlineExports: false,
  },
  output: {
    filename: '[name].js',
  },
};
