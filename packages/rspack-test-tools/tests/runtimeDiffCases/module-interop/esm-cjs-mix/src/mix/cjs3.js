typeof module.xxx;

it("should decorate commonjs module with node module decorator when access to module", function () {
	expect(__webpack_module__.children).toBeTruthy();
	expect(function () {
		__webpack_module__.exports = 1;
	}).not.toThrowError();
});
