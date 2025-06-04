/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./src/a.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "import",
		chunkFormat: "module",
		enabledChunkLoadingTypes: ["import"],
		environment: {
			arrowFunction: false,
			bigIntLiteral: false,
			const: false,
			destructuring: false,
			dynamicImport: false,
			dynamicImportInWorker: false,
			forOf: false,
			globalThis: false,
			module: false,
			optionalChaining: false,
			templateLiteral: false
		}
	},
	optimization: {
		runtimeChunk: {
			name: "bundle"
		},
		minimize: false,
		chunkIds: "named",
		moduleIds: "named",
		mangleExports: false,
		concatenateModules: true
	},

	devtool: false,
	target: "node",
	mode: "development"
};
