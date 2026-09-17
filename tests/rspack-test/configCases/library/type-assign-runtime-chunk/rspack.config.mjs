/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
    library: {
      name: 'MyLibraryRuntimeChunk',
      type: 'assign',
    },
  },
  target: 'web',
  optimization: {
    runtimeChunk: true,
  },
};
