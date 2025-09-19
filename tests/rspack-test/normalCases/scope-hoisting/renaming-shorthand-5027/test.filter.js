var supportsES6 = require("@rspack/test-tools/helper/legacy/supportsES6");
var supportDefaultAssignment = require("@rspack/test-tools/helper/legacy/supportDefaultAssignment");
var supportsObjectDestructuring = require("@rspack/test-tools/helper/legacy/supportsObjectDestructuring");
var supportsIteratorDestructuring = require("@rspack/test-tools/helper/legacy/supportsIteratorDestructuring");

module.exports = function (config) {
	return !config.minimize &&
		supportsES6() &&
		supportDefaultAssignment() &&
		supportsObjectDestructuring() &&
		supportsIteratorDestructuring();
};
