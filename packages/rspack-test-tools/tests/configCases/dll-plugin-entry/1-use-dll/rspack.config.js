var rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: './index.js',
	optimization: {
		moduleIds: "named"
	},
	plugins: [
		new rspack.DllReferencePlugin({
			manifest: require("../../../js/config/dll-plugin-entry/manifest0.json"), // eslint-disable-line node/no-missing-require
			name: "../0-create-dll/dll.js",
			scope: "dll",
			sourceType: "commonjs2"
		})
	]
};
