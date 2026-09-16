/** @type {import("@rspack/core").Configuration[]} */
export default {
  mode: 'production',
  entry: './index',
  optimization: {
    realContentHash: true,
  },
  output: {
    filename: '[fullhash].js',
  },
};
