const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");

module.exports = () => {
	return [FilteredStatus.PARTIAL_PASS, "check the consistency with webpack "];
};
