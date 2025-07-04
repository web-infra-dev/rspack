console.log("Hello Rspack with Module Federation");

// Import existing modules
import "./lib";
import {
	cjsExport,
	definedExport,
	functionExport,
	moduleExport,
	namedExport,
	default as testExportsDefault
} from "./test-exports";

import { VERSION, filter, map, uniq } from "lodash-es";
// Import external shared dependencies
import React from "react";
import ReactDOM from "react-dom/client";

import { createApiClient } from "./shared/api.js";
import { Button } from "./shared/components.js";
// Eager shared imports - loaded immediately and shared across federated modules
// Only import specific exports to enable better tree-shaking analysis
import { capitalize, formatDate } from "./shared/utils.js";

import { createApiClient as dynamicCreateApiClient } from "./shared/api.js";
import { Button as DynamicButton, Modal } from "./shared/components.js";
// Static imports for previously dynamic modules
import {
	capitalize as dynamicCapitalize,
	formatDate as dynamicFormatDate
} from "./shared/utils.js";

// CJS Test Package Usage Examples - Different import patterns for testing Module Federation
// Pattern 1: Direct require from alias
const cjsTestPackage = require("@cjs-test/legacy-utils");
console.log("CJS Test Package (via alias):", {
	name: cjsTestPackage.name,
	version: cjsTestPackage.version,
	formatPath: cjsTestPackage.formatPath("/test/path")
});

// Pattern 2: Import specific modules from the CJS test package
const {
	formatPath: cjsFormatPath,
	constants: cjsConstants
} = require("cjs-modules/legacy-utils");
const {
	processArray: cjsProcessArray,
	dataUtils: cjsDataUtils
} = require("cjs-modules/data-processor");

// Pattern 3: Test federated CJS modules (if exposed to other apps)
const testCjsFederated = async () => {
	try {
		// This would be used by other federated apps to consume our CJS modules
		const { formatPath } = await import("./cjs-test");
		console.log(
			"Federated CJS test module loaded:",
			formatPath("/federated/path")
		);
	} catch (error) {
		console.log(
			"Federated CJS import not available in current context:",
			error.message
		);
	}
};

// Import CommonJS modules for interoperability testing - ONLY import some exports to test unused detection
const {
	processArray,
	dataUtils
	// Intentionally NOT importing: createProcessor, filterArray, reduceArray, DataProcessor, DEFAULT_OPTIONS
} = require("./cjs-modules/data-processor.js");
const {
	formatPath,
	constants
	// Intentionally NOT importing: FileManager, readFileSync, validateFile, getSelf
} = require("./cjs-modules/legacy-utils.js");

// Import CommonJS modules for testing CommonJS tracking in the plugin
const legacyUtils = require("./cjs-modules/legacy-utils.js");
const dataProcessor = require("./cjs-modules/data-processor.js");

// Pure CommonJS require usage - NO ES6 imports, only require()
const pureCjsHelper = require("./cjs-modules/pure-cjs-helper.js");

// Test the pure module.exports = { ... } pattern
const moduleExportsPattern = require("./cjs-modules/module-exports-pattern.js");

console.log("Test exports:", {
	namedExport,
	functionExport: functionExport(),
	cjsExport,
	moduleExport,
	definedExport,
	default: testExportsDefault
});

// Test React shared dependency
console.log("React version:", React.version);
const reactElement = React.createElement(
	"div",
	{ id: "test" },
	"Hello from React!"
);
console.log("Created React element:", reactElement);

// Test lodash shared dependency
console.log("Lodash version:", VERSION);
const sampleData = [1, 2, 3, 4, 5];
const doubled = map(sampleData, n => n * 2);
console.log("Lodash map result:", doubled);
const filtered = filter(sampleData, n => n > 2);
console.log("Lodash filter result:", filtered);

// Test specific shared module exports (tree-shakeable)
console.log("Formatted date:", formatDate(new Date()));
console.log("Capitalized text:", capitalize("hello world"));

const button = new Button("Eager Button", () =>
	console.log("Eager button clicked!")
);
console.log("Created button:", button);

const client = createApiClient("https://api.example.com", {
	Authorization: "Bearer token"
});
console.log("Created API client:", client);

// Static usage of previously dynamic modules
console.log("Static formatted date:", dynamicFormatDate(new Date()));
console.log("Static capitalized text:", dynamicCapitalize("static hello"));

const staticButton = new DynamicButton("Static Button", () =>
	console.log("Static button clicked!")
);
const staticModal = new Modal("Static Modal", "This is a static loaded modal");
staticModal.open();

const staticClient = dynamicCreateApiClient("https://static.api.example.com", {
	Authorization: "Bearer static-token"
});
console.log("Created static API client:", staticClient);

// Test CJS test package usage through different patterns
console.log("CJS Test Package Usage:");
console.log("- Format path via alias:", cjsFormatPath("/test/from/alias"));
console.log("- Constants via alias:", cjsConstants);
console.log(
	"- Process array via modules:",
	cjsProcessArray([1, 2, 3], x => x * 3)
);
console.log("- Data utils via modules:", cjsDataUtils.sum([10, 20, 30]));

// Execute federated test
testCjsFederated();

// Test CommonJS modules usage - ONLY use imported exports to test unused detection
const testData = [1, 2, 3, 4, 5];
const processedData = processArray(testData, x => x * 2);
console.log("Processed array:", processedData);

console.log("Data utils sum:", dataUtils.sum(testData));
console.log("Data utils average:", dataUtils.average(testData));

// NOTE: NOT using createProcessor, DataProcessor, filterArray, reduceArray to test unused detection

const filePath = "/some/path/to/file.txt";
console.log("Formatted path:", formatPath(filePath));
console.log("Path constants:", constants);

// NOTE: NOT using FileManager, readFileSync, validateFile to test unused detection

// Test CommonJS modules usage to trigger ConsumeShared tracking
console.log("CommonJS Legacy Utils:", {
	name: legacyUtils.name,
	version: legacyUtils.version,
	join: legacyUtils.join("test", "path")
});

console.log("CommonJS Data Processor:", {
	version: dataProcessor.version,
	sum: dataProcessor.dataUtils.sum([1, 2, 3, 4, 5]),
	processArray: dataProcessor.processArray([1, 2, 3], x => x * 2)
});

// Test pure CommonJS helper - ONLY use some exports to test unused detection
console.log("Pure CommonJS Helper:", {
	info: pureCjsHelper.info,
	generateId: pureCjsHelper.generateId(),
	helpers: {
		timestamp: pureCjsHelper.helpers.timestamp(),
		random: pureCjsHelper.helpers.random()
	},
	constants: pureCjsHelper.CONSTANTS
});

// NOTE: NOT using hashString, validateInput, processData, DataValidator, createValidator
// These should appear as unused exports in the analysis

// Test the pure module.exports = { ... } pattern - ONLY use selected exports
console.log("Module Exports Pattern Test:", {
	info: moduleExportsPattern.moduleInfo,
	// Math utilities
	sum: moduleExportsPattern.calculateSum([1, 2, 3, 4, 5]),
	average: moduleExportsPattern.calculateAverage([10, 20, 30]),
	minMax: moduleExportsPattern.findMinMax([5, 2, 8, 1, 9]),
	// String utilities
	slugified: moduleExportsPattern.slugify("Hello World Test"),
	capitalized: moduleExportsPattern.capitalize("hello"),
	// Formatting
	currency: moduleExportsPattern.formatCurrency(1234.56),
	percentage: moduleExportsPattern.formatPercentage(0.75),
	// Date utilities
	formattedDate: moduleExportsPattern.formatDate(new Date()),
	isWeekend: moduleExportsPattern.isWeekend(new Date()),
	// Validation
	isEmailValid: moduleExportsPattern.isEmail("test@example.com"),
	isUrlValid: moduleExportsPattern.isUrl("https://example.com"),
	isEmpty: moduleExportsPattern.isEmpty(""),
	// Constants
	mathConstants: moduleExportsPattern.MATH_CONSTANTS,
	httpStatus: moduleExportsPattern.HTTP_STATUS.OK,
	// DataStore usage
	dataStore: (() => {
		const store = moduleExportsPattern.createDataStore();
		store.set("test", "value");
		return {
			hasTest: store.has("test"),
			testValue: store.get("test"),
			json: store.toJSON()
		};
	})()
});

// NOTE: Intentionally NOT using some exports to test tree-shaking:
// - truncate, transformData, filterData, groupBy
// - isEmail in some contexts, DataStore constructor directly
// These should appear as unused in the tree-shaking analysis
