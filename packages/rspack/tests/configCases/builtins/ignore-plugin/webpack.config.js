const rspack = require("@rspack/core");

module.exports = {
	plugins: [
		new rspack.IgnorePlugin({
			resourceRegExp: /^\.\/b$/,
		}),
		new rspack.IgnorePlugin({
			resourceRegExp: /^\.\/c$/,
			contextRegExp: /moment$/
		}),
		new rspack.IgnorePlugin({
			resourceRegExp: /^\.\/d$/,
			contextRegExp: /test-ignore$/
		}),
	]
};
