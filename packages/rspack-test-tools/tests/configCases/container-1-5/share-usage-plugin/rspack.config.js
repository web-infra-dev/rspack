const { ModuleFederationPlugin } = require("@rspack/core").container;
// TODO: Import ShareUsagePlugin when available in JavaScript API
// const { ShareUsagePlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index.js",
	optimization: {
		minimize: false,
		sideEffects: false
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "share_usage_test",
			exposes: {
				"./utils": "./utils.js",
				"./components": "./components.js"
			},
			shared: {
				"lodash-es": {
					singleton: true,
					requiredVersion: "*",
					shareKey: "lodash-es"
				},
				react: {
					singleton: true,
					requiredVersion: "^18.0.0",
					shareKey: "react"
				}
			}
		})
		// TODO: Add ShareUsagePlugin when available in JavaScript API
		// new ShareUsagePlugin({
		//   filename: "share-usage.json"
		// })
	]
};
