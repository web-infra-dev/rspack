const fs = require("fs");
const path = require("path");

it("html code gen entry order", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	let polyFillIndex = htmlContent.search("polyfill.js");
	let mainIndex = htmlContent.search("main.js");
	// should keep the order as entry in `rspack.config.js`
	expect(polyFillIndex < mainIndex).toBe(true);
});
