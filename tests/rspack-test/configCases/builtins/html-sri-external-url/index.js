const fs = require("fs");
const path = require("path");

it("should skip SRI for external CDN URLs", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	
	// External CDN script should NOT have integrity attribute
	expect(htmlContent).toContain('src="https://cdn.jsdelivr.net/npm/react@18/umd/react.production.min.js"');
	expect(htmlContent).not.toMatch(/react@18.*?integrity=/);
	
	// External CDN script should NOT have crossorigin attribute (only added with integrity)
	const reactScriptMatch = htmlContent.match(/<script[^>]*src="https:\/\/cdn\.jsdelivr\.net\/npm\/react@18[^>]*>/);
	if (reactScriptMatch) {
		expect(reactScriptMatch[0]).not.toContain('crossorigin');
	}
	
	// Local bundled script SHOULD have integrity and crossorigin
	expect(htmlContent).toMatch(/<script[^>]*src="[^"]*main\.js"[^>]*integrity="sha384-[^"]*"/);
	expect(htmlContent).toMatch(/<script[^>]*src="[^"]*main\.js"[^>]*crossorigin="anonymous"/);
});
