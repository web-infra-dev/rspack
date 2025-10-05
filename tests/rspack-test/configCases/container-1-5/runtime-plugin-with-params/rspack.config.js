const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    filename: "[name].js",
    uniqueName: "runtime-plugin-with-params"
  },
  entry: {
    main: [
      "./index.js",
      "./App.js"
    ]
  },
  plugins: [
    new ModuleFederationPlugin({
      name: "container-runtime-plugin-with-params",
      filename: "container.js",
      library: { type: "commonjs-module" },
      runtimePlugins: [
        "./plugin.js",
        ["./plugin-with-params.js", {
          testParam1: "value1",
          testParam2: 123,
          testParam3: true
        }],
        ["./complex-plugin.js", {
          nestedConfig: {
            enabled: true,
            options: [1, 2, 3]
          },
          callbackName: "onPluginLoad"
        }]
      ]
    })
  ]
};
