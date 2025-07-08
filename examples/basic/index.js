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

// Removed external dependencies - using only local shared modules

// Eager shared imports - loaded immediately and shared across federated modules
// Only import specific exports to enable better tree-shaking analysis
import { formatDate, capitalize } from "./shared/utils.js";
import { Button } from "./shared/components.js";
import { createApiClient } from "./shared/api.js";

// Import new shared modules with various patterns
import { join, basename, utils as pathUtils, createPathHandler } from "./shared/commonjs-module.js";
import { version, calculate, DataProcessor, createLogger, default as mixedDefault } from "./shared/mixed-exports.js";

// Import module.exports as default export  
import moduleExportsLib from "./shared/module-exports.js";

// Import fake CommonJS local module
import fakeLib from "./fake-node-module/index.js";
import { validateEmail, capitalize, createLogger: createFakeLogger, constants } from "./fake-node-module/index.js";

// Static imports for previously dynamic modules
import {
	formatDate as dynamicFormatDate,
	capitalize as dynamicCapitalize
} from "./shared/utils.js";
import { Button as DynamicButton, Modal } from "./shared/components.js";
import { createApiClient as dynamicCreateApiClient } from "./shared/api.js";

console.log("Test exports:", {
	namedExport,
	functionExport: functionExport(),
	cjsExport,
	moduleExport,
	definedExport,
	default: testExportsDefault
});

// Test local shared dependencies only

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

// Test new shared modules with various import patterns
console.log("=== Testing CommonJS Module ===");
console.log("Path join:", join("/home", "user"));
console.log("Basename:", basename("/home/user/file.txt"));
console.log("Path normalize:", pathUtils.normalize("../home//user/./file.txt"));

const pathHandler = createPathHandler("/base/path");
console.log("Path handler resolve:", pathHandler.resolve("file.txt"));

console.log("=== Testing Mixed Exports ===");
console.log("Version:", version);
console.log("Calculate 5 + 3:", calculate(5, 3, 'add'));
console.log("Calculate 10 / 2:", calculate(10, 2, 'divide'));

const processor = new DataProcessor("TestProcessor");
processor.add("item1").add("item2");
console.log("Processed data:", processor.process());

const logger = createLogger("APP");
logger.info("Testing logger functionality");

console.log("Mixed default export:", mixedDefault);

console.log("=== Testing Module Exports ===");
console.log("Add 15 + 25:", moduleExportsLib.add(15, 25));
console.log("Multiply 7 * 8:", moduleExportsLib.multiply(7, 8));
console.log("Math square of 9:", moduleExportsLib.math.square(9));
console.log("Constants PI:", moduleExportsLib.constants.PI);

const calculator = moduleExportsLib.createCalculator(10);
console.log("Calculator result:", calculator.add(5).multiply(2).result());

console.log("=== Testing Fake CommonJS Node Module ===");
console.log("Validate email:", validateEmail("test@example.com"));
console.log("Capitalize:", capitalize("hello world"));
console.log("Constants:", constants);

const fakeLogger = createFakeLogger("FAKE");
fakeLogger.info("Testing fake CommonJS module");

// Test accessing properties from whole module
console.log("Fake lib debounce:", typeof fakeLib.debounce);
console.log("Fake lib slugify:", fakeLib.slugify("Hello World Test"));
