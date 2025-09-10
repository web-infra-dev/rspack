const fs = require("fs");
const path = require("path");

it("[minify-parser]: import attributes should be preserved", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "importAttributes.js"), "utf-8");
	expect(content).toContain('import o from"./a.json"with{type:"json"}');
});
