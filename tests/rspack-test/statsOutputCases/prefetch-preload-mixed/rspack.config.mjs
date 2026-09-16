/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  stats: {
    all: false,
    chunkRelations: true,
    chunks: true,
  },
};
