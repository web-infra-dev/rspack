const supportsRequireInModule = require("@rspack/test-tools/helper/legacy/supportsRequireInModule");

module.exports = config => {
	return !config.module || supportsRequireInModule();
};
