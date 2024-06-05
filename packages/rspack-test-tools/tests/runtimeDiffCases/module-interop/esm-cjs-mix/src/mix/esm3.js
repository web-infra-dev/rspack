import "./bbb";

module.__esModule = 1;

it("should decorate esm and commonjs mix with harmony module decorator when assign to module._esModule", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
