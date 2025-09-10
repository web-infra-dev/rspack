import "./bbb";

it("should decorate esm and commonjs mix with ES modules decorator when assign to module.exports", function () {
	expect(function () {
		module.exports = 1;
	}).toThrowError();
});
