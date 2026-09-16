/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  resolve: {
    exportsFields: ['a', 'b'],
  },
};
