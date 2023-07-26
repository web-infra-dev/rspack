const supportsWebAssembly = require("../../../helpers/supportsWebAssembly");
const supportsFeature = require("webassembly-feature");

module.exports = function (config) {
	// CompileError: WebAssembly.instantiate(): Compiling function #0 failed: memory instruction with no memory @+27
	return false
	// return supportsWebAssembly() && supportsFeature.simd();
};
