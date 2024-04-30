const HmrPlugin = require("@rspack/core").HotModuleReplacementPlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	mode: "development",
	plugins: [new HmrPlugin()],
	module: {
		rules: [
			{
				loader: "./loader.js"
			}
		]
	}
};
