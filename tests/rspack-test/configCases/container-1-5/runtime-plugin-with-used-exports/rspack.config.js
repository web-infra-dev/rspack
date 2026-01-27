const path = require("path");
const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		providedExports: true,
		usedExports: true,
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "runtime-plugin-with-used-exports",
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
				path.resolve(__dirname, "runtime-plugin.js")
			]
		})
	]
};
