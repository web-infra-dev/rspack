const fs = require("fs");
const path = require("path");

it("should not add a doctype to a template without one", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.toLowerCase()).not.toContain("<!doctype");
	expect(htmlContent).toContain('<div id="root"></div>');
});

it("should not add a doctype to a template without one when minify is enabled", () => {
	const htmlPath = path.join(__dirname, "./index.minified.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.toLowerCase()).not.toContain("<!doctype");
	expect(htmlContent).toContain('<div id="root"></div>');
});
