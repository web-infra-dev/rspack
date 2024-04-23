const fs = require("fs");
const path = require("path");

it("[minify-comment-some]: should keep Legal Comment and remove others", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");

	expect(content).toContain("Legal Comment");
	expect(content).toContain("@license Apache-2.0");
	expect(content).not.toContain("Foo Bar");
});
