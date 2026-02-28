const devtool = "eval-source-map";

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		devtool
	},
	{
		devtool,
		optimization: {
			moduleIds: "natural"
		}
	},
	{
		devtool,
		optimization: {
			moduleIds: "named"
		}
	},
	{
		devtool,
		optimization: {
			moduleIds: "deterministic"
		}
	},
	// TODO: support size module ids 
	// {
	// 	devtool,
	// 	optimization: {
	// 		moduleIds: "size"
	// 	}
	// },
	{
		entry: "./index?foo=bar",
		devtool,
		optimization: {
			moduleIds: "named"
		}
	},
	{
		entry: "./index.js?foo=bar",
		devtool,
		optimization: {
			moduleIds: "named"
		}
	},
	{
		entry: "alias",
		devtool,
		optimization: {
			moduleIds: "named"
		},
		resolve: {
			alias: {
				alias: "./index?foo=bar"
			}
		}
	},
	{
		entry: "pkg",
		devtool,
		optimization: {
			moduleIds: "named"
		}
	},
	{
		entry: "./index.ts?foo=bar",
		devtool,
		optimization: {
			moduleIds: "named"
		},
		module: {
			rules: [
				{
					test: /\.ts$/,
					loader: 'builtin:swc-loader',
				}
			]
		}
	}
];
