// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("../../../../").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			remoteType: "commonjs-module",
			remotes: {
				containerB: "../../1-container-full/dist/container.js"
			},
			shared: ["react"]
		})
	]
};
