module.xxx = 1;

it("should decorate commonjs module with node module decorator when assign to module.xxx", function () {
	expect(__webpack_module__.children).toBeTruthy();
	expect(function () {
		__webpack_module__.exports = 1;
	}).not.toThrowError();
});
