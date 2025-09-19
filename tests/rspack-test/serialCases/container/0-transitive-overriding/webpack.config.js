const { ModuleFederationPluginV1: ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "named",
		moduleIds: "named"
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container-with-shared",
			library: { type: "commonjs-module" },
			filename: "container-with-shared.js",
			exposes: ["./a", "./b", "./modules"],
			remotes: {
				"container-with-shared": "./container-with-shared.js"
			},
			shared: {
				"./shared": {
					shareKey: "shared",
					version: "1"
				}
			}
		})
	]
};
