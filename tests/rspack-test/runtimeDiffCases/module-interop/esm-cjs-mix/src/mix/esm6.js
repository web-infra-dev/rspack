import "./bbb";

module.xxx = 1;

it("should decorate esm and commonjs mix with ES modules decorator when assign to module.xxx", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
