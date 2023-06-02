it("should run", function () {});

it("should name define", function () {
	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8");

	expect(source).toMatch('define("NamedLibrary",');
	expect(source.includes("return __webpack_exports__")).toBe(true);
	expect(source.includes("return (function() {\nvar __webpack_modules__")).toBe(
		true
	);
});
