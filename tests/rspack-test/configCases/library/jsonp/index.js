it("should wrap output in jsonp callback", function () {
	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8");

	expect(source.startsWith("MyJsonpCallback(")).toBe(true);
	expect(source).toContain("return __webpack_exports__");
});

it("should pass exports to jsonp callback", function () {
	expect(global.__jsonpCapture).toBeDefined();
	expect(global.__jsonpCapture.answer).toBe(42);
});

export const answer = 42;
