/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'none',
  output: {
    filename: ({ chunk }) =>
      chunk.name === 'main' ? 'js/main.js' : '[name].bundle.js',
    library: {
      type: 'modern-module',
    },
  },
};
