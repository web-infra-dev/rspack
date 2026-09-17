/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'assign-properties', name: ['process', 'env'] },
  },
};
