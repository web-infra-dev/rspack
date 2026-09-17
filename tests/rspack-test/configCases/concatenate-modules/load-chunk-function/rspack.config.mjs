/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    entry1: './entry1',
    entry2: './entry2',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    concatenateModules: true,
  },
};
