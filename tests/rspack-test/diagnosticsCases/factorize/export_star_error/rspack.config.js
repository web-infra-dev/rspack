const { DefinePlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new DefinePlugin({
			"process.env.NODE_ENV": "development"
		})
	],
};
