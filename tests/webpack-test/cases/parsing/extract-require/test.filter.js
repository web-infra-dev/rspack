const { FilteredStatus } = require("../../../lib/util/filterUtil");

module.exports = () => [
	FilteredStatus.PARTIAL_PASS,
	"should not parse require in function arguments"
];
