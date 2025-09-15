const supportsOptionalChaining = require("@rspack/test-tools/helper/legacy/supportsOptionalChaining");

module.exports = function (config) {
	return !config.minimize && supportsOptionalChaining();
};
