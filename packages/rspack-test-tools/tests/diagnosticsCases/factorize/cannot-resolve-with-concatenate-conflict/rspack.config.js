/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: "production",
  entry: "./index.ts",
  resolve: {
    extensions: [".ts", ".js"],
  },
  optimization: {
    concatenateModules: true,
  },
  module: {
    rules: [
      {
        test: /\.(j|t)s$/,
        loader: "builtin:swc-loader",
        options: {
          target: "es5",
        },
        type: "javascript/auto",
      },
    ],
  },
};
