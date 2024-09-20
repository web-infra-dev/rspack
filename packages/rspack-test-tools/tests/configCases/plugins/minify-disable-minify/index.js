const fs = require("fs");
const path = require("path");

it("[minify-disable-minify]: should not minify code", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");

	expect(content).toContain("let a = process.env.a;");
	expect(content).toContain("console.log(a + process.env.b);");
});
