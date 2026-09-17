/** @type {import("@rspack/core").Configuration} */
export default {
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
