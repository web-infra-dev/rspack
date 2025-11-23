function usedLocalFunction() {
	return "This local function is used";
}

function unusedLocalFunction() {
	return "This local function is unused";
}

module.exports = {
	usedLocalFunction,
	unusedLocalFunction,
	constantValue: "test value",
	unusedConstant: "unused value",
	nestedObject: {
		prop1: "value1",
		prop2: "value2"
	}
};

module.exports.directProperty = "direct export";
