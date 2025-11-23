// Bootstrap file with actual test logic for CommonJS module export tracking

// Import from CommonJS module using exports.X pattern
const { formatDate, processData } = require("./cjs-exports-pattern");

// Import from CommonJS module using module.exports = { ... } pattern
const {
	calculateSum,
	formatCurrency
} = require("./cjs-module-exports-pattern");

// Import from CommonJS module with mixed patterns
const mixedModule = require("./cjs-mixed-pattern");
const { utilityA } = mixedModule;

// Validate the imports work
if (typeof formatDate !== "function") {
	throw new Error("formatDate should be a function");
}
if (typeof processData !== "function") {
	throw new Error("processData should be a function");
}
if (typeof calculateSum !== "function") {
	throw new Error("calculateSum should be a function");
}
if (typeof formatCurrency !== "function") {
	throw new Error("formatCurrency should be a function");
}
if (typeof utilityA !== "function") {
	throw new Error("utilityA should be a function");
}

// Validate the share-usage.json file
const validateShareUsage = require("./validate-share-usage");
validateShareUsage();

module.exports = {
	formatDate,
	processData,
	calculateSum,
	formatCurrency,
	utilityA
};
