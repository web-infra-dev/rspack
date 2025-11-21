const { HotModuleReplacementPlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		asyncWebAssembly: true
	},
	plugins: [new HotModuleReplacementPlugin()]
};
