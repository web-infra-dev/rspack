/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		entry: "./index",
		mode: "production",
		target: "web",
		output: {
			filename: "warning.pro-web.js"
		},
		stats: {
			assets: true,
			modules: true,
		}
	},
	{
		entry: "./index",
		mode: "production",
		target: "webworker",
		output: {
			filename: "warning.pro-webworker.js"
		},
		stats: {
			assets: true,
			modules: true,
		}
	},
	{
		entry: "./index",
		mode: "production",
		target: "node",
		output: {
			filename: "no-warning.pro-node.js"
		},
		stats: {
			assets: true,
			modules: true,
		}
	},
	{
		entry: "./index",
		mode: "development",
		devtool: 'eval',
		target: "web",
		output: {
			filename: "no-warning.dev-web.js"
		},
		stats: {
			assets: true,
			modules: true,
		}
	},
	{
		entry: "./index",
		mode: "development",
		devtool: 'eval',
		target: "node",
		output: {
			filename: "no-warning.dev-node.js"
		},
		stats: {
			assets: true,
			modules: true,
		}
	},
	{
		entry: "./index",
		mode: "development",
		devtool: 'eval',
		target: "web",
		performance: {
			maxAssetSize: 100
		},
		output: {
			filename: "no-warning.dev-web-with-limit-set.js"
		},
		stats: {
			assets: true,
			modules: true,
		}
	},
	{
		entry: "./index",
		mode: "production",
		target: "node",
		performance: {
			hints: "warning"
		},
		output: {
			filename: "warning.pro-node-with-hints-set.js"
		},
		stats: {
			assets: true,
			modules: true,
		}
	}
];
