it("should also work in a chunk", function () {
	var contextRequire = require.context(".", false, /two/);
	expect(contextRequire("./two")).toBe(2);
	var tw = "tw";
	expect(require("." + "/" + tw + "o")).toBe(2);
});
