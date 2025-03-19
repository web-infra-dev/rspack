var supportsWorker = require("../../../../dist/helper/legacy/supportsWorker");

module.exports = function (config) {
	return supportsWorker();
};
