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
						{ loader: "./simple-loader.js", parallel: { maxWorkers: 4 }, options: {} },
						{ loader: "./pitching-loader.js", parallel: { maxWorkers: 4 }, options: {} },
						{ loader: "./simple-async-loader.js", parallel: { maxWorkers: 4 }, options: {} }
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
			parallelLoader: true
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
							parallel: { maxWorkers: 4 },
							options: {}
						},
						{ loader: "./pitching-loader.js", parallel: { maxWorkers: 4 }, options: {} },
						{ loader: "./simple-async-loader.js", parallel: { maxWorkers: 4 }, options: {} }
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
			parallelLoader: true
		}
	},
	{
		module: {
			rules: [
				{
					test: /lib\.js$/,
					use: [
						{ loader: "./simple-loader.js", parallel: { maxWorkers: 4 }, options: {} },
						{
							loader: "builtin:test-pitching-loader",
							parallel: { maxWorkers: 4 },
							options: {}
						},
						{ loader: "./simple-async-loader.js", parallel: { maxWorkers: 4 }, options: {} }
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
			parallelLoader: true
		}
	},
	{
		module: {
			rules: [
				{
					test: /lib\.js$/,
					use: [
						{ loader: "./simple-loader.js", parallel: { maxWorkers: 4 }, options: {} },
						{ loader: "./pitching-loader.js", parallel: { maxWorkers: 4 }, options: {} },
						{
							loader: "builtin:test-simple-async-loader.js",
							parallel: false,
							options: {}
						}
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
			parallelLoader: true
		}
	}
];
