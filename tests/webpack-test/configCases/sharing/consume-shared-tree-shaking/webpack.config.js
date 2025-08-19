const { ProvideSharedPlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: "production",
  optimization: {
    usedExports: true,
  },
  plugins: [
    new ProvideSharedPlugin({
      provides: {
        "./module": {
          shareKey: "module",
          version: "1.0.0",
        },
      },
    }),
  ],
};
