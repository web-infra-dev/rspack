const fs = require("fs");
const path = require("path");

it("[minify-comment-default-false]: should not keep comments", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");
	expect(content.split("\n")).toHaveLength(1);
});
