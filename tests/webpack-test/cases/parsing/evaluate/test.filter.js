const { FilteredStatus } = require("../../../lib/util/filterUtil");

module.exports = () => [
	FilteredStatus.PARTIAL_PASS,
	"should not evaluate new RegExp for redefined RegExp"
];
