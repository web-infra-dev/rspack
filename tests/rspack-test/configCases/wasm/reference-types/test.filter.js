var supportsWebAssembly = require("@rspack/test-tools/helper/legacy/supportsWebAssembly");

module.exports = function (config) {
	const [major] = process.versions.node.split(".").map(Number);

	return major >= 18 && supportsWebAssembly();
};
