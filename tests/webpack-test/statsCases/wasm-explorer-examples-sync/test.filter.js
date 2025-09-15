
var supportsWebAssembly = require("@rspack/test-tools/helper/legacy/supportsWebAssembly");

module.exports = function (config) {
	return supportsWebAssembly();
};

