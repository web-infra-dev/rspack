const fs = require("fs");
const path = require("path");

it("should throw error", () => {
	expect(fs.readFileSync(path.join(__dirname, "fail.js"), "utf-8")).toContain(
		"ModuleParseError"
	);
});
