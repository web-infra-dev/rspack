const fs = require("fs");
const path = require("path");

it("html favicon with absolute path", () => {
	const faviconPath = path.join(__dirname, "./favicon-图标.ico");
	expect(fs.existsSync(faviconPath)).toBe(true);

	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent).toContain('<link href="/favicon-图标.ico" rel="icon">');
});
