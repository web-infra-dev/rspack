const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	entry: {
		a: "./a.js",
		b: "./b.js"
	},
	output: {
		filename: "[name].js"
	},
	devtool: "eval-source-map",
	externals: ["source-map"],
	externalsType: "commonjs",
	optimization: {
		concatenateModules: false
	},
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
