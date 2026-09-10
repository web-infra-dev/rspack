const { ModuleFederationPlugin } = require("@rspack/core").container;
module.exports = [false, true].map((concatenateModules) => ({
  target: "node",
  externals: { "./container.js": "commonjs ./container.js" },
  optimization: {
    concatenateModules,
    moduleIds: "deterministic",
    minimize: true,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: "selective_provider",
      filename: "container.js",
      library: { type: "commonjs-module" },
      exposes: { "./Shared": "./consumer.js", "./Payload": "./payload.js" },
      shared: {
        "shared-lib": {
          import: "./shared.js",
          version: "1.0.0",
          requiredVersion: false,
          singleton: true,
        },
      },
    }),
  ],
}));
