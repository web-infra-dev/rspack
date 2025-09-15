var supportsSpread = require("@rspack/test-tools/helper/legacy/supportsSpread");

module.exports = function (config) {
	return supportsSpread();
};

