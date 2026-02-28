const { dependencies } = require("./package.json");
const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "named",
		moduleIds: "named"
	},
	plugins: [
		new ModuleFederationPlugin({
			remoteType: "commonjs-module",
			remotes: {
				service: "../0-eager-shared/container.js"
			},
			shared: {
				"tiny-emitter": {
					eager: true,
					singleton: true,
					requiredVersion: dependencies["tiny-emitter"]
				}
			}
		})
	]
};
