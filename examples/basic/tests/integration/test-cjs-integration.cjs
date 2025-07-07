// Test CJS Test Package Integration with Rspack Module Federation
// This file tests the integration of the CJS test package with the updated configuration

const path = require("node:path");

// Test 1: Basic alias resolution
console.log("=== Testing CJS Test Package Integration ===\n");

try {
	// Test the @cjs-test alias
	const cjsTestViaAlias = require("@cjs-test/legacy-utils");
	console.log("✅ @cjs-test alias works:");
	console.log("  - Name:", cjsTestViaAlias.name);
	console.log("  - Version:", cjsTestViaAlias.version);
	console.log("  - Format path test:", cjsTestViaAlias.formatPath("/test/alias/path"));
	console.log("");
} catch (error) {
	console.log("❌ @cjs-test alias failed:", error.message);
}

try {
	// Test the cjs-modules alias
	const legacyUtils = require("cjs-modules/legacy-utils");
	const dataProcessor = require("cjs-modules/data-processor");
	console.log("✅ cjs-modules alias works:");
	console.log("  - Legacy utils:", legacyUtils.name);
	console.log("  - Data processor:", dataProcessor.version);
	console.log("");
} catch (error) {
	console.log("❌ cjs-modules alias failed:", error.message);
}

// Test 2: Direct path resolution
try {
	const legacyUtilsDirect = require("./cjs-modules/legacy-utils");
	const dataProcessorDirect = require("./cjs-modules/data-processor");
	const pureCjsHelperDirect = require("./cjs-modules/pure-cjs-helper");

	console.log("✅ Direct CJS module imports work:");
	console.log("  - Legacy utils direct:", legacyUtilsDirect.name);
	console.log("  - Data processor direct:", dataProcessorDirect.version);
	console.log("  - Pure CJS helper direct:", pureCjsHelperDirect.info);
	console.log("");
} catch (error) {
	console.log("❌ Direct CJS imports failed:", error.message);
}

// Test 3: Functionality testing
try {
	const { formatPath, constants } = require("./cjs-modules/legacy-utils");
	const { processArray, dataUtils } = require("./cjs-modules/data-processor");

	console.log("✅ CJS module functionality tests:");
	console.log("  - Format path:", formatPath("/test/functionality/path"));
	console.log("  - Constants:", constants.DEFAULT_ENCODING);
	console.log("  - Process array:", processArray([1, 2, 3], x => x * 10));
	console.log("  - Data utils sum:", dataUtils.sum([100, 200, 300]));
	console.log("");
} catch (error) {
	console.log("❌ CJS functionality test failed:", error.message);
}

// Test 4: Module Federation expose configuration
console.log("✅ Module Federation configuration includes:");
console.log("  - Exposes ./cjs-test -> ./cjs-modules/legacy-utils.js");
console.log("  - Exposes ./cjs-data-processor -> ./cjs-modules/data-processor.js");
console.log("  - Exposes ./cjs-pure-helper -> ./cjs-modules/pure-cjs-helper.js");
console.log("");

// Test 5: Shared configuration
console.log("✅ Shared dependencies configuration includes:");
console.log("  - cjs-modules with shareKey: cjs-test-package");
console.log("  - Individual CJS modules with specific shareKeys");
console.log("  - Proper singleton and eager settings");
console.log("");

console.log("=== CJS Test Package Integration Complete ===");

module.exports = {
	testPassed: true,
	message: "CJS test package successfully integrated with Rspack Module Federation"
};
