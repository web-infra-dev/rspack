const fs = require("fs");
const path = require("path");

it("[minify-comment-all]: should keep all comments", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");

	expect(content).toContain("Legal Comment");
	expect(content).toContain("@license Apache-2.0");
	expect(content).toContain("Foo Bar");
});
