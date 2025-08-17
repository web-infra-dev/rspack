// Bootstrap file with actual test logic for CommonJS module export tracking

it("should track CommonJS module exports in share-usage.json", async () => {
	// Import from CommonJS module using exports.X pattern
	const { formatDate, processData } = require("./cjs-exports-pattern");
	
	// Import from CommonJS module using module.exports = { ... } pattern
	const { calculateSum, formatCurrency } = require("./cjs-module-exports-pattern");
	
	// Import from CommonJS module with mixed patterns
	const mixedModule = require("./cjs-mixed-pattern");
	const { utilityA } = mixedModule;
	
	// Use the imports to prevent tree-shaking
	expect(typeof formatDate).toBe("function");
	expect(typeof processData).toBe("function");
	expect(typeof calculateSum).toBe("function");
	expect(typeof formatCurrency).toBe("function");
	expect(typeof utilityA).toBe("function");
	
	// Validate the share-usage.json file
	const validateShareUsage = require("./validate-share-usage");
	validateShareUsage();
});