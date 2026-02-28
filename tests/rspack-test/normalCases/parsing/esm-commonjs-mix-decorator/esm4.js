import "./bbb";

Object.keys(module);

it("should decorate esm and commonjs mix with ES modules decorator when access module", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
