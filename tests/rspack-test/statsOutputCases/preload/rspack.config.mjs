/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  stats: {
    all: false,
    assets: true,
    entrypoints: true,
    chunkGroupChildren: true,
    chunks: true,
  },
};
