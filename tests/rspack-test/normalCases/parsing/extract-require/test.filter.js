const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");

module.exports = () => [
	FilteredStatus.PARTIAL_PASS,
	"should not parse require in function arguments"
];
