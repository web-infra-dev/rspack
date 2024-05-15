/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: {
		extensionAlias: {
			".mjs": [".mts"]
		}
	}
};
