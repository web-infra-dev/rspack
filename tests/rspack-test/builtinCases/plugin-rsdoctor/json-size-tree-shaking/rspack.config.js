const {
  experiments: {
    RsdoctorPlugin
  }
} = require("@rspack/core");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  builtins: {
    define: {
      "process.env.NODE_ENV": "'production'",
    }
  },
  optimization: {
    providedExports: true,
    usedExports: true,
    sideEffects: false,
  },
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: true, // Enable module sources feature to collect JSON sizes
      chunkGraphFeatures: true
    })
  ]
};