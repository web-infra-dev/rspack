const {
	AngularWebpackPlugin,
	AngularWebpackLoaderPath
} = require('@ngtools/webpack');


const {NamedChunksPlugin} = require('@angular-devkit/build-angular/src/webpack/plugins/named-chunks-plugin');
const {OccurrencesPlugin} = require('@angular-devkit/build-angular/src/webpack/plugins/occurrences-plugin');
const {DedupeModuleResolvePlugin} = require('@angular-devkit/build-angular/src/webpack/plugins/dedupe-module-resolve-plugin');
/*
 *  @type {() => import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: 'production',
	devtool: false,
	target: ['web', 'es2015'],
	entry: {
		polyfills: ['zone.js'],
		main: ['./src/main.ts']
	},
	output: {
		'uniqueName': 'zackAngularCli',
		// 'hashFunction': 'xxhash64', // throws error
		// 'clean': true, // throws error
		// 'path': '/Users/zackarychapple/code/zackAngularCli/dist/zack-angular-cli',
		'publicPath': '',
		'filename': '[name].[contenthash:20].js',
		'chunkFilename': '[name].[contenthash:20].js',
		// 'crossOriginLoading': false, // throws error
		// 'trustedTypes': 'angular#bundler', // throws error
		// 'scriptType': 'module' // throws error
	},
	watch: false,
	// snapshot: {module: {hash: false}},
	// performance: {hints: false}, // throws error
	experiments: {
		// 'backCompat': false, // throws error
		// 'syncWebAssembly': true, // throws error
		'asyncWebAssembly': true
	},
	optimization: {
		runtimeChunk: false,
		splitChunks: {
			// 'maxAsyncRequests': null, // throws error
			'cacheGroups': {
				'default': {
					'chunks': 'async',
					'minChunks': 2,
					'priority': 10
				},
				'common': {
					'name': 'common',
					'chunks': 'async',
					'minChunks': 2,
					// 'enforce': true, // throws error
					'priority': 5
				},
				// 'vendors': false, // throws error
				// 'defaultVendors': false // throws error
			}
		}
	},
	builtins: {
		html: [{
			template: './src/index.html'
		}]
	},
	module: {
		parser: {
			javascript: {
				requireContext: false,
				// Disable auto URL asset module creation. This doesn't effect `new Worker(new URL(...))`
				// https://webpack.js.org/guides/asset-modules/#url-assets
				url: false,
			}
		},
		rules: [
			{
				"test": /\.?(svg|html)$/,
				"resourceQuery": /\?ngResource/,
				"type": "asset/source"
			},
			{ test: /[/\\]rxjs[/\\]add[/\\].+\.js$/, sideEffects: true },
			{
				test: /\.[cm]?[tj]sx?$/,
				resolve: { fullySpecified: false },
				exclude: [
					/[\\/]node_modules[/\\](?:core-js|@babel|tslib|web-animations-js|web-streams-polyfill|whatwg-url)[/\\]/
				],
				use: [
					{
						loader: '/Users/columferry/dev/nrwl/issues/rspack/rspack/node_modules/@angular-devkit/build-angular/src/babel/webpack-loader.js',
						options: {
							"cacheDirectory": "/Users/columferry/dev/nrwl/issues/rspack/rspack/.angular/cache/15.2.4/babel-webpack",
							"aot": true,
							"optimize": true,
							"supportedBrowsers": [
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
				loader: '/Users/columferry/dev/nrwl/issues/arspack/rspack/node_modules/@ngtools/webpack/src/ivy/index.js',
				exclude: [
					/[\\/]node_modules[/\\](?:css-loader|mini-css-extract-plugin|webpack-dev-server|webpack)[/\\]/
				]
			}
		]
	},
	plugins: [
		new NamedChunksPlugin(),
		new OccurrencesPlugin({
			aot: true,
			scriptsOptimization: false,
		}),
		// new DedupeModuleResolvePlugin({verbose: true}),
				new AngularWebpackPlugin({
			tsconfig: './tsconfig.app.json',
			'emitClassMetadata': false,
			'emitNgModuleScope': false,
			'jitMode': false,
			'fileReplacements': {},
			'substitutions': {},
			'directTemplateLoading': true,
			'compilerOptions': {
				'sourceMap': false,
				'declaration': false,
				'declarationMap': false,
				'preserveSymlinks': false
			},
			'inlineStyleFileExtension': 'scss'
		}),
	],
};
