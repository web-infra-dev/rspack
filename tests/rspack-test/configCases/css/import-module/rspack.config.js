const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [new rspack.HotModuleReplacementPlugin()],
	target: "web",
	mode: "development",
	module: {
		rules: [
			{
				test: /stylesheet\.js$/i,
				use: ["./a-pitching-loader.js"],
				type: "asset/source"
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},

};
