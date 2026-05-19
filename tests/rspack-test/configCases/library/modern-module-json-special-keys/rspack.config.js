/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    main: { import: './index.js', filename: 'bundle.mjs' },
    bundle: { import: './lib.js', filename: 'bundle.lib.mjs' },
  },
  output: {
    module: true,
    library: {
      type: 'modern-module',
    },
  },
};
