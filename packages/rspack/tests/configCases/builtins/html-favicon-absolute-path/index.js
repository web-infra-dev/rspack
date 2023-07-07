const fs = require("fs");
const path = require("path");

it("html favicon with absolute path", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes('<link rel="icon" href="/favicon.ico" />')).toBe(
		true
	);
});
