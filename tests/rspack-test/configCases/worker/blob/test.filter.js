const supportsWorker = require("@rspack/test-tools/helper/legacy/supportsWorker");
const supportsBlob = require("@rspack/test-tools/helper/legacy/supportsBlob");

module.exports = function (config) {
	return supportsWorker() && supportsBlob();
};
