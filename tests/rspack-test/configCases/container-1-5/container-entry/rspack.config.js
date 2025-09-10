const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container-file.js",
			library: {
				type: "commonjs-module"
			},
			exposes: {
				"./test": "./test",
				"./test2": ["./init-module", "./test2"],
				".": "./main"
			}
		})
	]
};
