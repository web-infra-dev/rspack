const { ModuleFederationPlugin } = require("@rspack/core").container;
// ShareUsagePlugin is automatically applied by Module Federation

module.exports = {
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},
	optimization: {
		sideEffects: true,
		usedExports: true,
		providedExports: true
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "commonjs-test",
			filename: "remoteEntry.js",
			exposes: {
				"./exports": "./index.js"
			},
			shared: {
				"./cjs-exports-pattern": {
					singleton: true,
					version: "1.0.0"
				},
				"./cjs-module-exports-pattern": {
					singleton: true,
					version: "1.0.0"
				},
				"./cjs-mixed-pattern": {
					singleton: true,
					version: "1.0.0"
				}
			}
		})
		// ShareUsagePlugin is automatically applied and generates share-usage.json
	]
};
