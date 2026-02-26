const fs = require("fs");
const path = require("path");

it("should add SRI for URLs under publicPath", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	
	// Script with publicPath should have integrity
	expect(htmlContent).toMatch(/<script[^>]*src="https:\/\/cdn\.example\.com\/assets\/main\.js"[^>]*integrity="sha384-[^"]*"/);
	expect(htmlContent).toMatch(/<script[^>]*src="https:\/\/cdn\.example\.com\/assets\/main\.js"[^>]*crossorigin="anonymous"/);
});
