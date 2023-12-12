const path = require("path");
/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	mode: "production",
	context: __dirname,
	entry: {
		main: "./index.js",
		sub: "./sub.js"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		minimize: false,
		mangleExports: false,
		moduleIds: "named",
		usedExports: true
	}
};
