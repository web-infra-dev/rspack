const { AngularWebpackPlugin } = require("@ngtools/webpack");
const minifyPlugin = require("@rspack/plugin-minify");
const HtmlWebpackPlugin = require("html-webpack-plugin");

const {
	DedupeModuleResolvePlugin,
} = require("@angular-devkit/build-angular/src/webpack/plugins/dedupe-module-resolve-plugin");
const BundleAnalyzerPlugin =
	require("webpack-bundle-analyzer").BundleAnalyzerPlugin;

const {
	NamedChunksPlugin,
} = require("@angular-devkit/build-angular/src/webpack/plugins/named-chunks-plugin");
const {
	OccurrencesPlugin,
} = require("@angular-devkit/build-angular/src/webpack/plugins/occurrences-plugin");
const path = require("path");

const is_webpack = process.env.IS_WEBPACK;

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap(
			"MyPlugin",
			(compilation, { normalModuleFactory }) => {
				// TODO: This is just a workaround to support set `_module.factoryMeta.sideEffects` on loaderContext , because I am not sure If we should add a new hook that called anyway,
				// This issue maybe addressed after discussing with rspack core member
				normalModuleFactory.hooks.afterResolve.tap("MyPlugin", () => {});
			},
		);
	}
}

/** @type {import('@rspack/cli').Configuration} */
module.exports = {
	devtool: false,
	target: ["web", "es2015"],
	entry: {
		polyfills: ["zone.js"],
		main: ["./src/main.ts"],
	},
	resolve: {
		extensions: [".ts", ".js"],
	},
	output: {
		uniqueName: "angular",
		hashFunction: "xxhash64",
		clean: true,
		path: path.resolve(__dirname, is_webpack ? "webpack-dist" : "dist"),
		publicPath: "",
		filename: "[name].[contenthash:20].js",
		chunkFilename: "[name].[contenthash:20].js",
		crossOriginLoading: false,
		trustedTypes: "angular#bundler",
		// 'scriptType': 'module' // throws error
	},
	watch: false,
	// snapshot: {module: {hash: false}},
	// performance: {hints: false}, // throws error
	experiments: {
		// 'backCompat': false, // throws error
		// 'syncWebAssembly': true, // throws error
		asyncWebAssembly: true,
	},
	optimization: {
		runtimeChunk: false,
		//swc has different behavior compare to terser,this lead output size inflate
		minimizer: [
			new minifyPlugin({
				minifier: "terser",
			}),
		],
		splitChunks: {
			// 'maxAsyncRequests': null, // throws error
			cacheGroups: {
				default: {
					chunks: "async",
					minChunks: 2,
					priority: 10,
				},
				common: {
					name: "common",
					chunks: "async",
					minChunks: 2,
					enforce: true,
					priority: 5,
				},
				// 'vendors': false, // throws error
				// 'defaultVendors': false // throws error
			},
		},
	},
	builtins: {
		html: [
			{
				template: "./src/index.html",
			},
		],
		codeGeneration: {
			keepComments: true,
		},
		define: {
			ngDevMode: false,
		},
	},
	module: {
		parser: {
			javascript: {
				requireContext: false,
				// Disable auto URL asset module creation. This doesn't effect `new Worker(new URL(...))`
				// https://webpack.js.org/guides/asset-modules/#url-assets
				url: false,
			},
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
				type: "asset/source",
			},
			{
				test: /\.?(scss)$/,
				resourceQuery: /\?ngResource/,
				use: [{ loader: "raw-loader" }, { loader: "sass-loader" }],
			},
			{ test: /[/\\]rxjs[/\\]add[/\\].+\.js$/, sideEffects: true },
			{
				test: /\.[cm]?[tj]sx?$/,
				exclude: [
					/[\\/]node_modules[/\\](?:core-js|@babel|tslib|web-animations-js|web-streams-polyfill|whatwg-url)[/\\]/,
				],
				use: [
					{
						loader: require.resolve(
							"@angular-devkit/build-angular/src/babel/webpack-loader.js",
						),
						options: {
							cacheDirectory: path.join(
								__dirname,
								"/.angular/cache/15.2.4/babel-webpack",
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
								"safari 15",
							],
						},
					},
				],
			},
			{
				test: /\.[cm]?tsx?$/,
				use: [{ loader: require.resolve("@ngtools/webpack/src/ivy/index.js") }],
				exclude: [
					/[\\/]node_modules[/\\](?:css-loader|mini-css-extract-plugin|webpack-dev-server|webpack)[/\\]/,
				],
			},
		],
	},
	plugins: [
		new DedupeModuleResolvePlugin(),
		new NamedChunksPlugin(),
		new OccurrencesPlugin({
			aot: true,
			scriptsOptimization: false,
		}),
		new MyPlugin(),
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
				preserveSymlinks: false,
			},
			inlineStyleFileExtension: "scss",
		}),
	],
};
