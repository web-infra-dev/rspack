/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: './index.js',
      runtime: false,
    },
  },
  output: {
    filename: '[name].js',
  },
};
