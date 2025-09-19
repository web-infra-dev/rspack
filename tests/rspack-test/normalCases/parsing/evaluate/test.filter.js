const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");

module.exports = () => [
	FilteredStatus.PARTIAL_PASS,
	"should not evaluate new RegExp for redefined RegExp"
];
