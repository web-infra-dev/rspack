const fs = require("fs");
const path = require("path");

it("html template content", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes("<div>production</div>")).toBe(true);
});

it("html template content", () => {
	const htmlPath = path.join(__dirname, "./index2.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes("<div>development</div>")).toBe(true);
});
