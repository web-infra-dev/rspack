/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    chunkLoadingGlobal: '__LOADED_CHUNKS__',
  },
  target: 'web',
};
