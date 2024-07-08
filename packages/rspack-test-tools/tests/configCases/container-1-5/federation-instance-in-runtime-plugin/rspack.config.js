const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		// concatenateModules: false,
		moduleIds: 'named'
	},
	plugins: [
		new ModuleFederationPlugin({
			runtimePlugins: ["./plugin.js"]
		})
	]
};
