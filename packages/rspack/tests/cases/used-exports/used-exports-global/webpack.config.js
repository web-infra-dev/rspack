const path = require("path");
/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	mode: "production",
	context: __dirname,
	entry: {
		main: "./index.js",
		sub: "./sub.js"
	},
	optimization: {
		minimize: false,
		mangleExports: false,
		moduleIds: "named",
		usedExports: "global"
	}
};
