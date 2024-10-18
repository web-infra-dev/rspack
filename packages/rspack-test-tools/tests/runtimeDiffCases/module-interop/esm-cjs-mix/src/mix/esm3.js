import "./bbb";

module.__esModule = 1;

it("should decorate esm and commonjs mix with ES modules decorator when assign to module._esModule", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
