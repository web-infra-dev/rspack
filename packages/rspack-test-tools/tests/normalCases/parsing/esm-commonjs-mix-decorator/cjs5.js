module.__esModule = 1;

it("should decorate commonjs module with node module decorator when assign to module.__esModule", function () {
	expect(__webpack_module__.children).toBeTruthy();
	expect(function () {
		__webpack_module__.exports = 1;
	}).not.toThrowError();
});
