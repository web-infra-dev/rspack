import "./bbb";

module = 1;

it("should decorate esm and commonjs mix with harmony module decorator when assign to module", function () {
	expect(function () {
		__webpack_module__.exports = 1;
	}).toThrowError();
});
