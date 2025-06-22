const rspack = require("@rspack/core");

/** @type {import('@rspack/cli').Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	mode: "development",
	devtool: false,
	output: {
		clean: true
	},
	optimization: {
		minimize: false, // Keep false for dev mode debugging
		usedExports: true,
		providedExports: true,
		sideEffects: true,
		// Enable all optimizations even in dev mode
		concatenateModules: false,
		innerGraph: true,
		// Additional optimizations for better tree-shaking analysis
		mangleExports: false,
		removeAvailableModules: true,
		removeEmptyChunks: true,
		mergeDuplicateChunks: true,
		moduleIds: "named",
		chunkIds: "named",
		// Tree-shaking related optimizations
		realContentHash: true
	},
	stats: {
		// Enable comprehensive stats output
		all: false,
		modules: true,
		moduleTrace: true,
		modulesSort: "name",
		reasons: true,
		usedExports: true,
		providedExports: true,
		chunks: false,
		chunkModules: true,
		chunkOrigins: true,
		depth: true,
		entrypoints: true,
		assets: false,
		hash: false,
		version: false,
		timings: false,
		builtAt: false,
		publicPath: false,
		outputPath: false,
		// Module Federation specific
		runtimeModules: false,
		runtime: false,
		// Additional useful information
		moduleAssets: true,
		nestedModules: true,
		source: false, // Don't include source code in stats
		// Error and warning information
		errors: false,
		errorDetails: false,
		warnings: false,
		// Performance and optimization info
		optimizationBailout: true,
		performance: true
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "basic_example",
			filename: "remoteEntry.js",

			// Share dependencies with other federated modules
			shared: {
				// Original shared modules
				"./shared/utils.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "utility-lib"
				},
				"./shared/components.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "component-lib"
				},
				"./shared/api.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "api-lib"
				},
				// New shared modules with various export patterns
				"./shared/commonjs-module.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "commonjs-lib"
				},
				"./shared/mixed-exports.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "mixed-exports-lib"
				},
				"./shared/module-exports.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "module-exports-lib"
				},
				// Fake CommonJS module as local shared
				"./fake-node-module/index.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "fake-commonjs-lib"
				}
			},

			// Remote modules this app can consume
			remotes: {
				remote_app: "remote_app@http://localhost:3001/remoteEntry.js"
			}
		})
	]
};
