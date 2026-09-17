/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'none',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: [{ loader: './loader-b.js' }, { loader: './loader-a.js' }],
      },
    ],
  },
};
