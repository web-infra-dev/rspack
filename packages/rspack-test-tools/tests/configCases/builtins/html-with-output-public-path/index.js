const fs = require("fs");
const path = require("path");

it("html plugin should respect output.publicPath", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes('<script src="/base/bundle0.js" defer>')).toBe(
		true
	);
});
