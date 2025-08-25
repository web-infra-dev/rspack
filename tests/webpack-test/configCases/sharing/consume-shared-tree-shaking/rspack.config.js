const { ModuleFederationPlugin } = require("@rspack/core").container;

/**
 * This test demonstrates tree-shaking with Module Federation and external usage preservation.
 *
 * The ShareUsagePlugin (automatically applied by ModuleFederationPlugin) will:
 * 1. Analyze which exports are used locally
 * 2. Load external-usage.json to see which exports are needed by external systems
 * 3. Generate share-usage.json with combined usage information
 * 4. FlagDependencyUsagePlugin reads this to preserve necessary exports during tree-shaking
 *
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	entry: './index.js',
  mode: "production",
  optimization: {
    usedExports: true,
    sideEffects: false,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: "app",
      filename: "remoteEntry.js",
      exposes: {
        "./module": "./module",
      },
      shared: {
        "./module": {
          shareKey: "module",
          version: "1.0.0",
          singleton: true,
        },
      },
    }),
  ],
};
