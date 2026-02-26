const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should add SRI for URLs under publicPath", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");

	// Script with publicPath should have integrity
	expect(htmlContent).toMatch(/integrity="sha384-[^"]*"[^>]*src="https:\/\/cdn\.example\.com\/assets\/bundle0\.js"/);
	expect(htmlContent).toMatch(/crossorigin[^>]*src="https:\/\/cdn\.example\.com\/assets\/bundle0\.js"/);
});
