/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'modern-module',
    },
  },
  optimization: {
    runtimeChunk: false,
    avoidEntryIife: true,
  },
};
