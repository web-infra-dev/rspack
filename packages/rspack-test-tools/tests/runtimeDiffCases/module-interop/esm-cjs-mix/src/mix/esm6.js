import "./bbb";

module.xxx = 1;

it("should decorate esm and commonjs mix with harmony module decorator when assign to module.xxx", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
