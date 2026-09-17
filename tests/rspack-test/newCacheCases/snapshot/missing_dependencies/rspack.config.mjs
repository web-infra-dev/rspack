/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  resolve: {
    alias: {
      alias_file: ['./file1', './file2'],
    },
  },
  cache: {
    type: 'persistent',
  },
};
