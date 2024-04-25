import "./bbb";

it("should decorate esm and commonjs mix with harmony module decorator when assign to module.exports", function () {
	expect(function () {
		module.exports = 1;
	}).toThrowError();
});
