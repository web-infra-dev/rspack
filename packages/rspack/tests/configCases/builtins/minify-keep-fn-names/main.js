const fs = require("fs");
const path = require("path");

it("should keep fn names", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "entry.js"), "utf-8");
	expect(content).toContain("function main");
});
