const { ModuleFederationPluginV1: ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    filename: "[name].js"
  },
	mode: "development",
	target: ["async-node", "es5"],
	plugins: [
		new ModuleFederationPlugin({
			name: "A",
      filename: "container-a.js",
			library: {
				type: "commonjs-module"
			},
			exposes: {
				".": "./a"
			},
			remoteType: "commonjs-module",
			remotes: {
        "A": "./container-a.js",
			}
		}),
	]
};
