const fs = require("fs");
const path = require("path");

it("html plugin should respect output.publicPath", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes('<script defer src="foorBar.js">')).toBeTruthy();
});
