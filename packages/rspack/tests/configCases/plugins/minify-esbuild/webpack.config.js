const minifyPlugin = require("@rspack/plugin-minify");
module.exports = {
	context: __dirname,
	target: "node",
	entry: {
		main: "./index.js"
	},
	optimization: {
		minimize: true,
		minimizer: [new minifyPlugin()]
	}
};
