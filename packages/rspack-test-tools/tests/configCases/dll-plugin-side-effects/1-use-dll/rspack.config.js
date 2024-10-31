var rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new rspack.DllReferencePlugin({
			manifest: require("../../../js/config/dll-plugin-side-effects/manifest0.json"), // eslint-disable-line node/no-missing-require
			name: "../0-create-dll/dll.js",
			scope: "dll",
			sourceType: "commonjs2"
		})
	]
};
