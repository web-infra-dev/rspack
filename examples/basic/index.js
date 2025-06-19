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
