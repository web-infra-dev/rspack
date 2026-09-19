const fs = require("fs");
const path = require("path");

const countDoctype = (content) =>
	content.toLowerCase().split("<!doctype").length - 1;

it("should keep the doctype when it is preceded by a comment", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(countDoctype(htmlContent)).toBe(1);
	expect(htmlContent).toContain("<!-- lorem ipsum -->");
	expect(htmlContent).toContain('<div id="root"></div>');
});

it("should keep a single doctype when minify is enabled", () => {
	const htmlPath = path.join(__dirname, "./index.minified.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(countDoctype(htmlContent)).toBe(1);
	expect(htmlContent).toContain('<div id="root"></div>');
});
