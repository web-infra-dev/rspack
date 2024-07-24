const webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	// plugin that intercepts __webpack_require__
	plugins: [new webpack.HotModuleReplacementPlugin()]
};
