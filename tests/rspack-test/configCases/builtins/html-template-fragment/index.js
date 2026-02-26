const fs = require("fs");
const path = require("path");

it("html template fragment (without doctype)", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.toLowerCase().includes("<!doctype")).toBe(false);
	expect(htmlContent.includes('<div id="app">')).toBe(true);
	expect(htmlContent.includes('<meta charset="utf-8">')).toBe(true);
	expect(htmlContent.includes('<div>scripts: "bundle0.js"</div>')).toBe(true);
});


