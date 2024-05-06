const { rspack } = require("@rspack/core");

module.exports = {
	plugins: [
		new rspack.DefinePlugin({
			"__DEV__": "😄"
		})
	],
}
