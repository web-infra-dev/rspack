const rspack = require("@rspack/core");

module.exports = {
  entry: { main: "./src/Three.js" },
  mode: "production",
  devtool: false,
  optimization: {
    minimize: false,
    usedExports: true,
    providedExports: true,
    sideEffects: false,
    concatenateModules: true,
    innerGraph: true,
    removeAvailableModules: true,
    removeEmptyChunks: true,
    mergeDuplicateChunks: true
  },
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: "threejs_test",
      filename: "remoteEntry.js",
      
      // Share the utility modules
      shared: {
        "./src/shared/utility.js": {
          singleton: true,
          eager: false,
          shareKey: "threejs-utility"
        },
        "./src/shared/config.js": {
          singleton: true,
          eager: false,
          shareKey: "threejs-config"
        },
        "./src/shared/math.js": {
          singleton: true,
          eager: false,
          shareKey: "threejs-math"
        }
      }
    })
  ]
};
