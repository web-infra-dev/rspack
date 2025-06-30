console.log("Hello Rspack with Module Federation");

// Import existing modules
import "./lib";
import {
	namedExport,
	functionExport,
	cjsExport,
	moduleExport,
	definedExport,
	default as testExportsDefault
} from "./test-exports";

// Import external shared dependencies
import React from "react";
import ReactDOM from "react-dom/client";
import { VERSION, map, filter, uniq } from "lodash-es";

// Eager shared imports - loaded immediately and shared across federated modules
// Only import specific exports to enable better tree-shaking analysis
import { formatDate, capitalize } from "./shared/utils.js";
import { Button } from "./shared/components.js";
import { createApiClient } from "./shared/api.js";

// Static imports for previously dynamic modules
import {
	formatDate as dynamicFormatDate,
	capitalize as dynamicCapitalize
} from "./shared/utils.js";
import { Button as DynamicButton, Modal } from "./shared/components.js";
import { createApiClient as dynamicCreateApiClient } from "./shared/api.js";

// Import CommonJS modules as npm packages - ONLY import some exports to test unused detection
const {
	processArray,
	dataUtils
	// Intentionally NOT importing: createProcessor, filterArray, reduceArray, DataProcessor, DEFAULT_OPTIONS
} = require("@cjs-test/data-processor");
const {
	formatPath,
	constants
	// Intentionally NOT importing: FileManager, readFileSync, validateFile, getSelf
} = require("@cjs-test/legacy-utils");

// Import CommonJS modules as packages for testing CommonJS + ConsumeShared tracking
const legacyUtils = require("@cjs-test/legacy-utils");
const dataProcessor = require("@cjs-test/data-processor");

// Pure CommonJS require usage - NO ES6 imports, only require()
const pureCjsHelper = require("@cjs-test/pure-cjs-helper");

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

// Test CommonJS modules as npm packages to trigger ConsumeShared tracking
console.log("CommonJS Legacy Utils Package:", {
	name: legacyUtils.name,
	version: legacyUtils.version,
	join: legacyUtils.join("test", "path")
});

console.log("CommonJS Data Processor Package:", {
	version: dataProcessor.version,
	sum: dataProcessor.dataUtils.sum([1, 2, 3, 4, 5]),
	processArray: dataProcessor.processArray([1, 2, 3], x => x * 2)
});

// Test pure CommonJS helper package - ONLY use some exports to test unused detection
console.log("Pure CommonJS Helper Package:", {
	info: pureCjsHelper.info,
	generateId: pureCjsHelper.generateId(),
	helpers: {
		timestamp: pureCjsHelper.helpers.timestamp(),
		random: pureCjsHelper.helpers.random()
	},
	constants: pureCjsHelper.CONSTANTS
});

// NOTE: NOT using hashString, validateInput, processData, DataValidator, createValidator
// These should appear as unused exports in the analysis and get tree-shaking macros if ConsumeShared works!
