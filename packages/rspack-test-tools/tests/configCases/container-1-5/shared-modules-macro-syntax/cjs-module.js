// CommonJS module specifically for testing macro syntax edge cases
function usedFunction() {
	return "This function is used";
}

function unusedFunction() {
	return "This function is unused";
}

// Object literal export pattern for testing comma handling
module.exports = {
	usedFunction,
	unusedFunction,
	constantValue: "test value",
	unusedConstant: "unused value",
	nestedObject: {
		prop1: "value1",
		prop2: "value2"
	}
};
