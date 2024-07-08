const fs = require("fs");
const path = require("path");

it("html plugin should public path auto works", () => {
	const htmlPath = path.join(__dirname, "./main_page/index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes('<script src="../bundle0.js" defer>')).toBe(true);
});
