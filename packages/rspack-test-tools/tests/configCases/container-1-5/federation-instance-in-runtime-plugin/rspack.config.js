const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		// concatenateModules: false,
		moduleIds: "named"
	},
	output: {
		filename: "someDir/[name].js",
		chunkFilename: "someDir/[name].js"
	},
	plugins: [
		new ModuleFederationPlugin({
			filename: "someDir/container.js",
			runtimePlugins: ["./plugin.js"]
		})
	]
};
