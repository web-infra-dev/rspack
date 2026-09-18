/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    a: './a/index.js',
    b: './b/index.js',
    main: './main/index.js',
  },
  output: {
    filename: '[name].js',
  },
};
