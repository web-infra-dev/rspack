const { FilteredStatus } = require("../../../lib/util/filterUtil")

/*
const supportsRequireInModule = require("../../../helpers/supportsRequireInModule");

module.exports = config => {
	return !config.module || supportsRequireInModule();
};

*/
module.exports = () => {return [FilteredStatus.PARTIAL_PASS, "https://github.com/web-infra-dev/rspack/issues/4350"]}

							