const { HotModuleReplacementPlugin } = require("@rspack/core");

module.exports = [
	{
		entry: "./a.js",
		output: {
			filename: "a.js",
			uniqueName: "a"
		},
		plugins: [new HotModuleReplacementPlugin()],
		target: "web"
	},
	{
		entry: "./b.js",
		output: {
			filename: "b.js",
			uniqueName: "b"
		},
		plugins: [new HotModuleReplacementPlugin()],
		target: "web"
	},
	{
		entry: "./index.js",
		output: {
			filename: "index.js"
		}
	}
];
