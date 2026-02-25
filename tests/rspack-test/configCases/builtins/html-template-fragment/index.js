const fs = require("fs");
const path = require("path");

it("html template fragment (without doctype)", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.toLowerCase().includes("<!doctype")).toBe(false);
	expect(htmlContent.includes('<div id="app"></div>')).toBe(true);
});
