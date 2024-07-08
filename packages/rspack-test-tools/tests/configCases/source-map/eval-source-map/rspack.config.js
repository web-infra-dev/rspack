const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	devtool: "eval-source-map",
	externals: ["source-map"],
	externalsType: "commonjs",
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
