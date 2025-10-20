// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPluginV1: ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			remoteType: "commonjs-module",
			manifest: false,
			remotes: {
				containerB: "../1-container-full/container.js"
			},
			shared: ["react"]
		})
	]
};
