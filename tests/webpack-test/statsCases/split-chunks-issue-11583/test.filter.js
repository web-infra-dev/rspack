const { FilteredStatus } = require("../../lib/util/filterUtil");

module.exports = () => {
	return [FilteredStatus.PARTIAL_PASS, "check the consistency with webpack "];
};
