const rspack = require("@rspack/core");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: {
		all: false,
		modules: true
	},
	plugins: [
		new rspack.IgnorePlugin({
			checkResource: (resource, request) => {
				if (resource.includes("zh") || resource.includes("globalIndex")) {
					return true;
				}
				return false;
			}
		})
	]
};
