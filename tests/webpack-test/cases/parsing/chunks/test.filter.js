const { FilteredStatus } = require("../../../lib/util/filterUtil")

/*
var supportsES6 = require("../../../helpers/supportsES6");

module.exports = function (config) {
	return supportsES6();
};

*/
module.exports = () => {return [FilteredStatus.PARTIAL_PASS, "https://github.com/web-infra-dev/rspack/issues/4304"]}

							