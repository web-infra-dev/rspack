const { rspack } = require("@rspack/core");
/**
 * @type {import('@rspack/core').RspackOptions[]}
 */
module.exports = [
	{
		module: {
			rules: [
				{
					test: /lib\.js$/,
					use: [
						{ loader: "./simple-loader.js", parallel: true, options: {} },
						{ loader: "./pitching-loader.js", parallel: false, options: {} },
						{ loader: "./simple-async-loader.js", parallel: true, options: {} }
					]
				}
			]
		},
		plugins: [
			new rspack.DefinePlugin({
				CONTEXT: JSON.stringify(__dirname)
			})
		],
		experiments: {
			parallelLoader: {
			maxWorkers: 8,
		}
		}
	},
	{
		module: {
			rules: [
				{
					test: /lib\.js$/,
					use: [
						{
							loader: "builtin:test-simple-loader",
							parallel: false,
							options: {}
						},
						{ loader: "./pitching-loader.js", parallel: true, options: {} },
						{ loader: "./simple-async-loader.js", parallel: true, options: {} }
					]
				}
			]
		},
		plugins: [
			new rspack.DefinePlugin({
				CONTEXT: JSON.stringify(__dirname)
			})
		],
		experiments: {
			parallelLoader: {
			maxWorkers: 8,
		}
		}
	},
	{
		module: {
			rules: [
				{
					test: /lib\.js$/,
					use: [
						{ loader: "./simple-loader.js", parallel: true, options: {} },
						{
							loader: "builtin:test-pitching-loader",
							parallel: false,
							options: {}
						},
						{ loader: "./simple-async-loader.js", parallel: true, options: {} }
					]
				}
			]
		},
		plugins: [
			new rspack.DefinePlugin({
				CONTEXT: JSON.stringify(__dirname)
			})
		],
		experiments: {
			parallelLoader: {
			maxWorkers: 8,
		}
		}
	},
	{
		module: {
			rules: [
				{
					test: /lib\.js$/,
					use: [
						"./simple-loader.js",
						"./pitching-loader.js",
						"builtin:test-simple-async-loader.js"
					]
				}
			]
		},
		plugins: [
			new rspack.DefinePlugin({
				CONTEXT: JSON.stringify(__dirname)
			})
		],
		experiments: {
			parallelLoader: {
			maxWorkers: 8,
		}
		}
	}
];
