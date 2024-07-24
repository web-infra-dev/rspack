const rspack = require("@rspack/core");
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	plugins: [
		new rspack.DefinePlugin({
			"typeof window": JSON.stringify("undefined")
		})
	]
}
