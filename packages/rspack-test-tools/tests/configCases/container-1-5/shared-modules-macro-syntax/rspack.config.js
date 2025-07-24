const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},
	optimization: {
		minimize: false,
		sideEffects: false
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "shared_modules_syntax_test",
			exposes: {
				"./cjs-module": "./cjs-module.js",
				"./esm-utils": "./esm-utils.js"
			},
			shared: {
				"./cjs-module": {
					singleton: true,
					requiredVersion: "*",
					shareKey: "cjs-module"
				},
				"./esm-utils": {
					singleton: true,
					requiredVersion: "*",
					shareKey: "esm-utils"
				}
			}
		})
	]
};
