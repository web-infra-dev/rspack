// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: ["./index.js", "./second.js"]
	},
	target: "async-node",
	output: {
		filename: "[name].js",
		uniqueName: "async-startup-multi-entry-node",
		chunkLoading: "async-node"
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "mf_multi_entry_async",
			library: { type: "commonjs-module" },
			filename: "container.js",
			experiments: { asyncStartup: true },
			// Minimal expose to activate federation runtime; unused at runtime.
			exposes: {
				"./stub": "./second.js"
			}
		})
	]
};
