const { FilteredStatus } = require("../../../lib/util/filterUtil");

/*
var supportsTemplateStrings = require("../../../helpers/supportsTemplateStrings");

module.exports = function (config) {
	return supportsTemplateStrings();
};

*/
module.exports = () => {
	return [
		FilteredStatus.PARTIAL_PASS,
		"amd require context",
		"require(String.raw``)"
	];
};
