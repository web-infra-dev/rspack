/** @type {import('@rspack/core').Configuration} */
export default {
  entry: './index',
  stats: {
    all: false,
    entrypoints: true,
    chunkGroups: true,
  },
};
