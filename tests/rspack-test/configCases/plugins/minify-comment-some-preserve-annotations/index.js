const fs = require("fs");
const path = require("path");

it("[minify-comment-some]: should keep annotation and remove others", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "a.js"), "utf-8");

	expect(content).toContain("/*#__PURE__*/");
	expect(content).toContain("/* @__PURE__ */");
});
