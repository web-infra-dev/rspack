/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  mode: "development",
  entry: {
    main: "./index.js"
  },
  optimization: {
    providedExports: true,
    usedExports: true,
    concatenateModules: true,
    minimize: false
  },
};
