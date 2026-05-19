const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should skip SRI for external URLs when publicPath is /", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");

	// External CDN script should NOT have integrity attribute
	expect(htmlContent).toContain('src="https://cdn.jsdelivr.net/npm/react@18/umd/react.production.min.js"');
	expect(htmlContent).not.toMatch(/react@18.*?integrity=/);

	// External CDN script should NOT have crossorigin attribute
	const reactScriptMatch = htmlContent.match(/<script[^>]*src="https:\/\/cdn\.jsdelivr\.net\/npm\/react@18[^>]*>/);
	if (reactScriptMatch) {
		expect(reactScriptMatch[0]).not.toContain('crossorigin');
		expect(reactScriptMatch[0]).not.toContain('integrity');
	}

	// Local bundled script SHOULD have integrity and crossorigin
	// Note: With publicPath="/", the src will be "/bundle0.js"
	expect(htmlContent).toMatch(/integrity="sha384-[^"]*"[^>]*src="\/bundle0\.js"/);
	expect(htmlContent).toMatch(/crossorigin[^>]*src="\/bundle0\.js"/);
});
