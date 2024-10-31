var path = require("path");
var rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: './index.js',
	module: {
		rules: [
			{
				oneOf: [
					{
						test: /\.abc\.js$/,
						loader: "../0-create-dll/g-loader.js",
						options: {
							test: 1
						}
					}
				]
			}
		]
	},
	optimization: {
		moduleIds: "named",
		concatenateModules: false,
	},
	resolve: {
		extensions: [".js", ".jsx"]
	},
	plugins: [
		new rspack.DllReferencePlugin({
			manifest: require("../../../js/config/dll-plugin/manifest0.json"), // eslint-disable-line node/no-missing-require
			name: "../0-create-dll/dll.js",
			context: path.resolve(__dirname, "../0-create-dll"),
			sourceType: "commonjs2",
		})
	]
};
