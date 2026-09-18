/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  resolve: {
    extensionAlias: {
      '.mjs': ['.mts'],
    },
  },
};
