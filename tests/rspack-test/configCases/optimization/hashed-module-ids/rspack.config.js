var webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
// matches webpack's usage for backwards compatibility: https://webpack.js.org/configuration/optimization/#optimizationmoduleids
module.exports = {
	optimization: {
		moduleIds: false
	},
	plugins: [new webpack.HashedModuleIdsPlugin()]
};
