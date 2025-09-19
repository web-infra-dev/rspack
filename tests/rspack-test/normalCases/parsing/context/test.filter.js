const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");

module.exports = () => [
	FilteredStatus.PARTIAL_PASS,
	"not support moduleName of Circular dependency warning yet"
];
