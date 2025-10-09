const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    filename: "[name].js",
    uniqueName: "runtime-plugin-with-params"
  },
  plugins: [
    new ModuleFederationPlugin({
      name: "container-runtime-plugin-with-params",
      filename: "container.js",
      library: { type: "commonjs-module" },
				shared: {
				react: {
					version: false,
					requiredVersion: false,
					singleton: true,
					strictVersion: false,
					version: "0.1.2"
				}
			},
      runtimePlugins: [
        ["./plugin-with-params.js", {
          'custom-params': {
						msg: 'custom-params',
					}
        }],
      ]
    })
  ]
};
