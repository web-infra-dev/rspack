const fs = require("fs");
const path = require("path");

it("[minify-ascii-only]: chunk a should be minified", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");

	expect(content).toContain("\\u4F60\\u597D\\uFF0Cworld!");
});
