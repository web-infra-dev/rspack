/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    module: true,
    filename: '[name].mjs',
  },
  target: ['web', 'es2020'],
  optimization: {
    minimize: true,
    runtimeChunk: 'single',
  },
};
