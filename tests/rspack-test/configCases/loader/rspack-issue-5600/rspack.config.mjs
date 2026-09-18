/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'none',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: [{ loader: './loader-b.mjs' }, { loader: './loader-a.mjs' }],
      },
    ],
  },
};
