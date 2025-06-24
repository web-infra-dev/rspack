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
	resolve: {
		// Add alias for the CJS test package
		alias: {
			"@cjs-test": require
				.resolve("./cjs-modules/package.json")
				.replace("/package.json", ""),
			"cjs-modules": require
				.resolve("./cjs-modules/package.json")
				.replace("/package.json", "")
		}
	},
	optimization: {
		minimize: false, // Keep false for dev mode debugging
		usedExports: true,
		providedExports: true,
		sideEffects: false,
		// Enable all optimizations even in dev mode
		concatenateModules: false,
		innerGraph: true,
		// Additional optimizations for better tree-shaking analysis
		mangleExports: true,
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

			// Expose CJS test modules for other federated apps to consume
			exposes: {
				"./cjs-test": "./cjs-modules/legacy-utils.js",
				"./cjs-data-processor": "./cjs-modules/data-processor.js",
				"./cjs-pure-helper": "./cjs-modules/pure-cjs-helper.js",
				"./cjs-module-exports": "./cjs-modules/module-exports-pattern.js"
			},

			// Share dependencies with other federated modules
			shared: {
				// Share utilities - actually imported by the app
				"./shared/utils": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "utility-lib"
				},
				"./shared/components": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "component-lib"
				},
				"./shared/api": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "api-lib"
				},

				// Share CJS test package modules
				"cjs-modules": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "cjs-test-package",
					shareScope: "cjs-testing"
				},
				"./cjs-modules/legacy-utils.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "cjs-legacy-utils"
				},
				"./cjs-modules/data-processor.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "cjs-data-processor"
				},
				"./cjs-modules/pure-cjs-helper.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "cjs-pure-helper"
				},
				"./cjs-modules/module-exports-pattern.js": {
					singleton: true,
					eager: false,
					requiredVersion: false,
					shareKey: "cjs-module-exports"
				},

				// Share external libraries that are actually used
				react: {
					singleton: true,
					requiredVersion: "^18.2.0",
					eager: false,
					shareKey: "react",
					shareScope: "default"
				},
				"react-dom": {
					singleton: true,
					requiredVersion: "^18.2.0",
					eager: false,
					shareKey: "react-dom",
					shareScope: "default"
				},
				"lodash-es": {
					singleton: true,
					requiredVersion: "^4.17.21",
					eager: false,
					shareKey: "lodash-es",
					shareScope: "default"
				}
			},

			// Remote modules this app can consume
			remotes: {
				remote_app: "remote_app@http://localhost:3001/remoteEntry.js",
				cjs_test_remote: "cjs_test@http://localhost:3002/remoteEntry.js"
			}
		})

		// NOTE: CommonJS modules accessed via require() cannot be made ConsumeShared
		// They are ProvideShared but consumed directly, which is a current limitation
	]
};
