var supportsES6 = require("@rspack/test-tools/helper/legacy/supportsES6");

module.exports = function (config) {
	return supportsES6();
};

