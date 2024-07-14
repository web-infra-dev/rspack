const fs = require("fs");
const path = require("path");

it("[minify-exclude-css]: chunk a should be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.css"), "utf-8");
	expect(content).not.toMatch("\n");
});

it("[minify-exclude-css]: chunk b should not be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "b.css"), "utf-8");
	expect(content).toMatch("\n");
});
