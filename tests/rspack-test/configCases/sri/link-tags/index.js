const fs = require("fs");
const path = require("path");

it("should not process link tags that are not modulepreload, preload, or stylesheet", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent).toContain('<script crossorigin defer integrity');
	expect(htmlContent).toContain('<link href="https://example.com" rel="dns-prefetch">');
	expect(htmlContent).toContain('<link href="https://example.com" rel="preconnect">');
	expect(htmlContent).toContain('<link href="https://example.com" rel="prefetch">');
});
