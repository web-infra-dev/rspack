const fs = require("fs");
// should preserve comments in generated code

it("should preserve comments in generated code", function () {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).toContain("// should preserve comments in generated code");
});
