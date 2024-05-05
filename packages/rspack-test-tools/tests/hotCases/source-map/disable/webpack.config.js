/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: { 
    main: './index.js',
  },
  devtool: false,
  externalsPresets: {
    node: true,
  },
  node: {
    __dirname: false,
  }
}