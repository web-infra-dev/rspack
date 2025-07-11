// Test the CommonJS exports macro positioning fix

// Individual assignments (should be: /* @common:if [...] */ exports.func1 = function() { return 'func1'; }; /* @common:endif */)
exports.func1 = function () {
	return "func1";
};
exports.func2 = function () {
	return "func2";
};

// Variable assignments (should wrap right-hand side)
const fn1 = exports.func1;
const fn2 = exports.func2;

// Object literal (should remain unchanged)
module.exports = {
	prop1: "value1",
	prop2: "value2",
	method1: function () {
		return "method1";
	}
};
