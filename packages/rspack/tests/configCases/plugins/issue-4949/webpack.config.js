const rspack = require("@rspack/core");

/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	plugins: [
		false && new rspack.DefinePlugin({
			STRING: '"string"',
		})
	]
};
