it("should run", function () {});

it("should name define", function () {
	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8");

	expect(source).toMatch('define("NamedLibrary",');
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(source.includes("return __rspack_exports")).toBe(true);
		expect(source.includes("return (() => {\nvar __rspack_modules")).toBe(true);
	} else {
		expect(source.includes("return __webpack_exports__")).toBe(true);
		expect(source.includes("return (() => {\nvar __webpack_modules__")).toBe(
			true
		);
	}
});
