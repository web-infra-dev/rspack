const {
	AngularWebpackPlugin,
	AngularWebpackLoaderPath
} = require('@ngtools/webpack');
/*
 *  @type {() => import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: 'production',
	devtool: false,
	target: ['web', 'es2015'],
	entry: {
		main: ['./src/main.ts'],
		polyfills: ['zone.js']
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
		runtimeChunk: true,
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
		rules: [
			{
				test: /\.ts$/,
				use: {
					loader: AngularWebpackLoaderPath
				},
				exclude: [
					/[\\/]node_modules[/\\](?:css-loader|mini-css-extract-plugin|webpack-dev-server|webpack)[/\\]/,
				],
				type: 'typescript',
			},
			{
				test: /\.?(svg|html)$/,
				// Only process HTML and SVG which are known Angular component resources.
				resourceQuery: /\?ngResource/,
				type: 'asset/source',
			},
		]
	},
	plugins: [
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
