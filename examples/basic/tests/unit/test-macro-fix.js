// Test file to verify macro positioning fix
exports.func1 = function () {
	return "func1";
};
exports.func2 = function () {
	return "func2";
};
module.exports.func3 = function () {
	return "func3";
};

// Test variable assignment
const fn = exports.func1;

// Test object literal
module.exports = {
	prop1: "value1",
	prop2: "value2"
};
