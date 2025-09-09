const fs = require("fs");
const path = require("path");

it("[minify-include]: chunk a should be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");
	expect(content).not.toMatch("\n");
});

it("[minify-include]: chunk b should not be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "b.js"), "utf-8");
	expect(content).toMatch("\n");
});
