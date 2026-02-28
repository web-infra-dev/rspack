module.exports = 1;

it("should not decorate commonjs module", function () {
	expect(__webpack_module__.children).toBeFalsy();
});
