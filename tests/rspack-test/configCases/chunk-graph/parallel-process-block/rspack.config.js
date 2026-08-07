/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    first: './index.js',
    // This entry gives the shared named chunk a different available-modules set.
    second: './module-b.js',
  },
  output: {
    filename: '[name].js',
  },
};
