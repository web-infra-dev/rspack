const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");

module.exports = () => [
	FilteredStatus.PARTIAL_PASS,
	"require.include",
	"support amd"
];
