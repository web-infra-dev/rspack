var ProvidePlugin = require("../../../../").ProvidePlugin;
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ProvidePlugin({
			"xxx.yyy": "aaa"
		})
	]
};
