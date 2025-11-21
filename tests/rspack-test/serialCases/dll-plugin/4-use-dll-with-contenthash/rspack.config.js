var webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		moduleIds: "named"
	},
	plugins: [
		new webpack.DllReferencePlugin({
			manifest: require("../../../js/config/dll-plugin/manifest0.json"),
			name: "../0-create-dll-with-contenthash/dll.js",
			scope: "dll",
			sourceType: "commonjs2",
			extensions: [".js", ".jsx"]
		})
	]
};
