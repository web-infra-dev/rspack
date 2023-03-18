/**
 * format the targetDir
 */
exports.formatTargetDir = function (targetDir) {
	return targetDir.trim().replace(/\/+$/g, "");
};
