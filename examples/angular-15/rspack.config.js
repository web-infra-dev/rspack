const { AngularWebpackPlugin } = require("@ngtools/webpack");

// const {DedupeModuleResolvePlugin} = require('@angular-devkit/build-angular/src/webpack/plugins/dedupe-module-resolve-plugin');
const {
	NamedChunksPlugin
} = require("@angular-devkit/build-angular/src/webpack/plugins/named-chunks-plugin");
const {
	OccurrencesPlugin
} = require("@angular-devkit/build-angular/src/webpack/plugins/occurrences-plugin");
const path = require("path");
/** @type {import('@rspack/cli').Configuration} */
module.exports = {
	mode: "production",
	devtool: false,
	target: ["web", "es2015"],
	entry: {
		polyfills: ["zone.js"],
		main: ["./src/main.ts"]
	},
	resolve: {
		extensions: [".ts", ".js"]
	},
	output: {
		uniqueName: "zackAngularCli",
		// 'hashFunction': 'xxhash64', // throws error
		clean: true,
		// path: "./dist",
		publicPath: "",
		filename: "[name].[contenthash:20].js",
		chunkFilename: "[name].[contenthash:20].js",
		crossOriginLoading: false,
		trustedTypes: "angular#bundler"
		// 'scriptType': 'module' // throws error
	},
	watch: false,
	// snapshot: {module: {hash: false}},
	// performance: {hints: false}, // throws error
	experiments: {
		// 'backCompat': false, // throws error
		// 'syncWebAssembly': true, // throws error
		asyncWebAssembly: true
	},
	optimization: {
		runtimeChunk: false,
		minimize:true,
		splitChunks: {
			// 'maxAsyncRequests': null, // throws error
			cacheGroups: {
				default: {
					chunks: "async",
					minChunks: 2,
					priority: 10
				},
				common: {
					name: "common",
					chunks: "async",
					minChunks: 2,
					// 'enforce': true, // throws error
					priority: 5
				}
				// 'vendors': false, // throws error
				// 'defaultVendors': false // throws error
			}
		}
	},
	builtins: {
		html: [
			{
				template: "./src/index.html"
			}
		]
	},
	module: {
		parser: {
			javascript: {
				requireContext: false,
				// Disable auto URL asset module creation. This doesn't effect `new Worker(new URL(...))`
				// https://webpack.js.org/guides/asset-modules/#url-assets
				url: false
			}
		},
		rules: [
			// {
			// 	// ! THIS IS FOR TESTING ANGULAR HMR !
			// 	include: [path.resolve("./src/main.ts")],
			// 	use: [
			// 		{
			// 			loader: require.resolve(
			// 				"@angular-devkit/build-angular/src/webpack/plugins/hmr/hmr-loader.js"
			// 			)
			// 		}
			// 	]
			// },
			{
				test: /\.?(svg|html)$/,
				resourceQuery: /\?ngResource/,
				type: "asset/source"
			},
			{
				test: /\.?(scss)$/,
				resourceQuery: /\?ngResource/,
				use: [{ loader: "raw-loader" }, { loader: "sass-loader" }]
			},
			{ test: /[/\\]rxjs[/\\]add[/\\].+\.js$/, sideEffects: true },
			{
				test: /\.[cm]?[tj]sx?$/,
				exclude: [
					/[\\/]node_modules[/\\](?:core-js|@babel|tslib|web-animations-js|web-streams-polyfill|whatwg-url)[/\\]/
				],
				use: [
					{
						loader: require.resolve(
							"@angular-devkit/build-angular/src/babel/webpack-loader.js"
						),
						options: {
							cacheDirectory: path.join(
								__dirname,
								"/.angular/cache/15.2.4/babel-webpack"
							),
							aot: true,
							optimize: true,
							supportedBrowsers: [
								"chrome 111",
								"chrome 110",
								"edge 111",
								"edge 110",
								"firefox 111",
								"firefox 102",
								"ios_saf 16.3",
								"ios_saf 16.2",
								"ios_saf 16.1",
								"ios_saf 16.0",
								"ios_saf 15.6",
								"ios_saf 15.5",
								"ios_saf 15.4",
								"ios_saf 15.2-15.3",
								"ios_saf 15.0-15.1",
								"safari 16.3",
								"safari 16.2",
								"safari 16.1",
								"safari 16.0",
								"safari 15.6",
								"safari 15.5",
								"safari 15.4",
								"safari 15.2-15.3",
								"safari 15.1",
								"safari 15"
							]
						}
					}
				]
			},
			{
				test: /\.[cm]?tsx?$/,
				use: [{ loader: require.resolve("@ngtools/webpack/src/ivy/index.js") }],
				exclude: [
					/[\\/]node_modules[/\\](?:css-loader|mini-css-extract-plugin|webpack-dev-server|webpack)[/\\]/
				]
			}
		]
	},
	plugins: [
		// TODO: Add this back after https://github.com/web-infra-dev/rspack/issues/2619 lands
		// new DedupeModuleResolvePlugin(),
		new NamedChunksPlugin(),
		new OccurrencesPlugin({
			aot: true,
			scriptsOptimization: false
		}),
		new AngularWebpackPlugin({
			tsconfig: "./tsconfig.app.json",
			emitClassMetadata: false,
			emitNgModuleScope: false,
			jitMode: false,
			fileReplacements: {},
			substitutions: {},
			directTemplateLoading: true,
			compilerOptions: {
				sourceMap: false,
				declaration: false,
				declarationMap: false,
				preserveSymlinks: false
			},
			inlineStyleFileExtension: "scss"
		})
	]
};
