const path = require("path");

module.exports = {
	output: {
		pathinfo: "verbose"
	},
	module: {
		rules: [
			{
				test: path.resolve(__dirname, "lib.js"),
				sideEffects: false
			}
		]
	},
	optimization: {
		sideEffects: true,
		usedExports: true,
		providedExports: true,
		concatenateModules: false
	}
};
