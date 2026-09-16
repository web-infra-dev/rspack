/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  stats: {
    all: false,
    assets: true,
    ids: true,
    entrypoints: true,
    chunkGroupChildren: true,
    chunkRelations: true,
    chunks: true,
  },
};
