const rspack = require("../../packages/rspack");
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	target: "node",
	devtool: false,
	entry: {
		a: "./a",
		a2: "./a2",
		b: "./b",
		main: "./index"
	},
	optimization: {
		minimize: true
	},
	plugins: [
		new rspack.SwcJsMinimizerRspackPlugin({
			test: [/a\d?\.js/],
			exclude: [/a\.js/]
		})
	]
};
