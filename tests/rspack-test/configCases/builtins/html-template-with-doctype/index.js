const fs = require("fs");
const path = require("path");

it("html template with doctype", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.toLowerCase().includes("<!doctype html>")).toBe(true);
	expect(htmlContent.includes('<div id="app"></div>')).toBe(true);
});
