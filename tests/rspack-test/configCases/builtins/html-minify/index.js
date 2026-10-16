const fs = require("fs");
const path = require("path");

it("html minify", () => {
	const htmlPath = path.join(__dirname, "./index.html");
	const htmlContent = fs.readFileSync(htmlPath, "utf-8");
	expect(htmlContent.includes("</script></head><body></body></html>")).toBe(true);
});

it("should preserve adjacent inline script execution boundaries", () => {
	const htmlContent = fs.readFileSync(path.join(__dirname, "index.html"), "utf-8");
	const scripts = Array.from(
		htmlContent.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g),
		match => match[1]
	);
	const first = scripts.findIndex(script => script.includes("first script"));

	expect(first).toBeGreaterThanOrEqual(0);
	expect(scripts[first]).toContain("throw");
	expect(scripts[first + 1]).toContain("secondScript");
	expect(scripts[first + 1]).toContain("document.currentScript");
});
