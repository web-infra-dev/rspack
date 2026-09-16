/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  resolve: {
    byDependency: {
      esm: {
        extensions: ['.bar', '...'],
      },
    },
  },
};
