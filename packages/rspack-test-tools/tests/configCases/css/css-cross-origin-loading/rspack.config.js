/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    crossOriginLoading: "anonymous",
  },
  entry: "./index.js",
  target: "web",
  module: {
    rules: [
      {
        test: /\.css$/,
        type: "css/module"
      }
    ]
  },
  experiments: {
    css: true
  },
};
