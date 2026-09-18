it("should set correct options on js files", function() {
	expect(require("./loader.mjs!./index.js")).toEqual({
		minimize: true,
		jsfile: true
	});
});
it("should set correct options on other files", function() {
	expect(require("./loader.mjs!./txt.txt")).toEqual({
		minimize: true
	});
});
