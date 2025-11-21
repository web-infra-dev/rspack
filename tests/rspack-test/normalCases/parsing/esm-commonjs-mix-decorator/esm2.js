import "./bbb";

module = 1;

it("should decorate esm and commonjs mix with ES modules decorator when assign to module", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
