const { DefinePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		target: "web",
		plugins: [
			// TODO: support type: "module" injection
			new DefinePlugin({
				MODULE_FLAG: "undefined"
			})
		]
	},
	{
		output: {
			filename: "[name].bundle1.js"
		},
		target: "web",
		optimization: {
			runtimeChunk: "single"
		},
		plugins: [
			// TODO: support type: "module" injection
			new DefinePlugin({
				MODULE_FLAG: "undefined"
			})
		]
	},
	{
		target: "web",
		experiments: {
			outputModule: true
		},
		plugins: [
			// TODO: support type: "module" injection
			new DefinePlugin({
				MODULE_FLAG: "\"module\""
			})
		]
	},
	{
		target: "web",
		output: {
			filename: "[name].bundle3.mjs"
		},
		optimization: {
			runtimeChunk: "single"
		},
		experiments: {
			outputModule: true
		},
		plugins: [
			// TODO: support type: "module" injection
			new DefinePlugin({
				MODULE_FLAG: "\"module\""
			})
		]
	}
];
