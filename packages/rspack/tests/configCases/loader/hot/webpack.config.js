const HmrPlugin = require("@rspack/core").HotModuleReplacementPlugin;

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
