
var supportsWorker = require("@rspack/test-tools/helper/legacy/supportsWorker");

module.exports = function (config) {
	// return supportsWorker() && config.target !== "async-node";
	// FIXME: not stable on CI
	return false;
};
