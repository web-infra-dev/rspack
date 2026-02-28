var DefinePlugin = require("@rspack/core").DefinePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		environment: {
			bigIntLiteral: true
		}
	},
	plugins: [
		new DefinePlugin({
			BIGINT: BigInt("9007199254740993"),
			ZERO_BIGINT: BigInt(0)
		})
	]
};
