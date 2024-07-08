const path = require("path");
/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	mode: "production",
	context: __dirname,
	entry: {
		main: "./index.js",
		sub: "./sub.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		minimize: false,
		mangleExports: false,
		moduleIds: "named",
		usedExports: "global"
	}
};
