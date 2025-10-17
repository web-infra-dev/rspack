const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js",
		uniqueName: "async-startup-no-dynamic"
	},
	experiments: {
		mfAsyncStartup: true
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			library: { type: "commonjs-module" },
			filename: "container.js",
			remotes: {
				containerA: "../0-container-full/container.js"
			},
			shared: ["react"],
			experiments: {
				asyncStartup: true
			}
		})
	]
};
