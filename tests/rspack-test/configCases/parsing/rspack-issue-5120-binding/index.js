const fs = require("fs");
const path = require("path");

it("should throw error", () => {
	expect(fs.readFileSync(path.join(__dirname, "fail.js"), "utf-8")).toContain(
		"JavaScript parse error: Unexpected token `let`. Expected let is reserved in const, let, class declaration"
	);
});
