const { ModuleFederationPlugin, ShareUsagePlugin } =
	require("@rspack/core").container;
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index.js",
	optimization: {
		minimize: false,
		sideEffects: false,
		usedExports: true
	},
	output: {
		filename: "[name].js",
		uniqueName: "share_usage_inter_dependency_test"
	},
	plugins: [
		new ShareUsagePlugin({
			filename: "share-usage.json"
		}),
		new ModuleFederationPlugin({
			name: "share_usage_inter_dependency_test",
			filename: "remoteEntry.js",
			exposes: {
				"./store": "./store.js"
			},
			shared: {
				redux: {
					singleton: true,
					requiredVersion: "^4.2.0",
					shareKey: "redux"
				},
				"@reduxjs/toolkit": {
					singleton: true,
					requiredVersion: "^1.9.5",
					shareKey: "@reduxjs/toolkit"
				}
			}
		})
	]
};
