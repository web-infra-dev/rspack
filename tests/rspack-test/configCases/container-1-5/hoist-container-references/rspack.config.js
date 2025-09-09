const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		splitChunks: {
			chunks: "all"
		},
		moduleIds: "named"
	},
	plugins: [new ModuleFederationPlugin({})]
};
