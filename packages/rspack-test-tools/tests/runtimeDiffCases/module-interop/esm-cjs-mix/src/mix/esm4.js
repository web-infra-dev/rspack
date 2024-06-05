import "./bbb";

it("should decorate esm and commonjs mix with harmony module decorator when access module", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
