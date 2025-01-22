
it("((function (define) {...}) (define)) should compile well", function () {
	// Throws an exception message if it is a compilation exception:
	// (function (__webpack_require__.amdD) {
	// SyntaxError: Unexpected token '.'

	expect(() => require('./lib')).toThrow('define cannot be used indirect');
});

