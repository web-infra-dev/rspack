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
			name: "container",
			filename: "container.js",
			library: { type: "commonjs-module" },
			exposes: {
				"./emitter": {
					name: "emitter",
					import: "./emitter.js"
				}
			},
			shared: {
				...dependencies
			}
		})
	]
};
