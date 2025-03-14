const supportsWebAssembly = require("../../../helpers/supportsWebAssembly");

// This test is disabled because it fails in webpack. The reason is the wasm file try to access memory without memory initialization.
// see: https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/test/cases/wasm/v128/test.filter.js
module.exports = function (config) {
	// CompileError: WebAssembly.instantiate(): Compiling function #0 failed: memory instruction with no memory @+27
	return false
	// return supportsWebAssembly() && supportsFeature.simd();
};
