const { rspack } = require("@rspack/core");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	plugins: [
		new rspack.DefinePlugin({
			DEFINE_VAR: "1 2 3"
		})
	],
	optimization: {
		concatenateModules: true
	}
};
