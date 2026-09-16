/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: './index.js',
      runtime: 'runtime',
    },
  },
  output: {
    filename: '[name].js',
  },
};
