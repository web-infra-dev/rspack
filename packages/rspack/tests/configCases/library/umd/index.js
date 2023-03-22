it("should run", function () {
	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8");

	expect(source.includes("return __webpack_exports__")).toBe(true);
});
