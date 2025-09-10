import "./bbb";

it("should decorate esm and commonjs mix with ES modules decorator when Object.defineProperty(module, 'exports', xxx);", function () {
	expect(function () {
		Object.defineProperty(module, "exports", 1);
	}).toThrowError();
});
