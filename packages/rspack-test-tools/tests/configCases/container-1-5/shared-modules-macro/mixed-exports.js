// Module with mixed CJS and ESM patterns (for interop testing)
const { processCjsData } = require("./cjs-module.js");
import { usedUtil } from "./esm-utils.js";

// Named export for test
export const namedExport = "named value";

// CJS style exports
exports.mixedFunction = function(data) {
	return processCjsData(data) + " + " + usedUtil();
};

exports.cjsStyleExport = "CJS style value";

// Also try module.exports patterns
module.exports.moduleExportsProp = {
	value: "module.exports property",
	timestamp: Date.now()
};

module.exports.interopFunction = function() {
	return "interop function result";
};

// Unused mixed exports
exports.unusedMixedFunction = function() {
	return "unused mixed function";
};

module.exports.unusedModuleExportsProp = "unused property";

// Default export for test
export default {
	defaultValue: "default export value"
};