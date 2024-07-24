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
		"https://github.com/web-infra-dev/rspack/issues/4304, https://github.com/web-infra-dev/rspack/issues/4313",
		"require(String.raw``)"
	];
};
