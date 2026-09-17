/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    bundle0: './require-entry-point',
    a: './entry-point',
    b: ['./entry-point2'],
  },
  output: {
    filename: '[name].js',
  },
};
