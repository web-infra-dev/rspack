var supportsDefaultAssignment = require("@rspack/test-tools/helper/legacy/supportDefaultAssignment");
var supportsObjectDestructuring = require("@rspack/test-tools/helper/legacy/supportsObjectDestructuring");

module.exports = function (config) {
	return supportsDefaultAssignment() && supportsObjectDestructuring();
};
