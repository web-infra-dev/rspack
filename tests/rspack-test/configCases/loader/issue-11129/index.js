it("should handle exports-loader with inline-style options", function () {
	const inlineResult = require("./lib.js?inline");
	const objectResult = require("./lib.js?object");

	expect(inlineResult).toBeDefined();
	expect(objectResult).toBeDefined();
});
