// CommonJS module with multiple export patterns for tree-shaking macro testing
function usedFunction() {
	return "This function is used";
}

function unusedFunction() {
	return "This function is unused";
}

const usedConstant = "used constant";
const unusedConstant = "unused constant";

// Helper function for complex patterns
function createObject() {
	return {
		id: Math.random(),
		timestamp: Date.now()
	};
}

// Circular reference pattern
function getSelf() {
	return module.exports;
}

// Process function similar to tree-shake-macro test
function processCjsData(data) {
	return "processed: " + data;
}

// Export patterns that should get tree-shaking macros
exports.usedFunction = usedFunction;
exports.unusedFunction = unusedFunction;
exports.usedConstant = usedConstant;
exports.unusedConstant = unusedConstant;
exports.createObject = createObject;
exports.getSelf = getSelf;
exports.processCjsData = processCjsData;

// Mixed export patterns
module.exports.mixedExport = "mixed export value";
module.exports.anotherMixed = {
	prop: "value",
	nested: {
		deep: "property"
	}
};

module.exports.unusedCjsFunction = function() {
	return "unused cjs function";
};