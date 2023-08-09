const fs = require("fs");
const path = require("path");

it("[minify-comment-false]: should remove all comments by default", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");

	expect(content).not.toContain("Legal Comment");
	expect(content).not.toContain("@license Apache-2.0");
	expect(content).not.toContain("Foo Bar");
});
