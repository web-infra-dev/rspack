
var supportsWorker = require("@rspack/test-tools/helper/legacy/supportsWorker");

module.exports = function (config) {
	return supportsWorker();
};
