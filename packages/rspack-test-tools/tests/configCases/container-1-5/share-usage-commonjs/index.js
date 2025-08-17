// Test for CommonJS module export tracking in share-usage.json

// Import from CommonJS module using exports.X pattern
const { formatDate, processData } = require("./cjs-exports-pattern");

// Import from CommonJS module using module.exports = { ... } pattern
const { calculateSum, formatCurrency } = require("./cjs-module-exports-pattern");

// Import from CommonJS module with mixed patterns
const mixedModule = require("./cjs-mixed-pattern");
const { utilityA } = mixedModule;

// Use the imports to prevent tree-shaking
console.log("Testing CommonJS export tracking:");
console.log("formatDate:", typeof formatDate);
console.log("processData:", typeof processData);
console.log("calculateSum:", typeof calculateSum);
console.log("formatCurrency:", typeof formatCurrency);
console.log("utilityA:", typeof utilityA);

module.exports = {
	formatDate,
	processData,
	calculateSum,
	formatCurrency,
	utilityA
};