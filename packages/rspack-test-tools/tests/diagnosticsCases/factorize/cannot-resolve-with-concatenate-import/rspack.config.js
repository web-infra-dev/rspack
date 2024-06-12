/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: "production",
  entry: "./index.ts",
  resolve: {
    extensions: [".tsx", ".ts", ".js"],
  },
  optimization: {
    minimize: true,
    concatenateModules: true,
  },
  module: {
    rules: [
      {
        test: /\.(j|t)s$/,
        loader: "builtin:swc-loader",
        options: {
          target: "es5",
          jsc: {
            externalHelpers: true,
          },
          env: {
            mode: "usage",
            coreJs: "3.37.1",
          },
        },
        type: "javascript/auto",
      },
    ],
  },
};
