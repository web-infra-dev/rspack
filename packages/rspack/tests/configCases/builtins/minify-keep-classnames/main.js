const fs = require("fs");
const path = require("path");

it("should keep class names", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "entry.js"), "utf-8");
	expect(content).toContain("class KeepClass");
});
