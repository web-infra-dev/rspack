var supportsWorker = require("../../../../dist/helper/legacy/supportsWorker");

module.exports = function (config) {
	// TODO: port https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/dependencies/HarmonyAcceptDependency.js#L86-L98
	return supportsWorker() && false;
};
