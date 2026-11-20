import "./entry.css";

const fs = require("fs");
const path = require("path");

it("should preserve inherited conditions and intersect media queries for external @import rules in CSS chunks", () => {
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");

	expect(css).toContain('@import url("https://example.com/shared.css");');
	expect(css).toContain('@import url("https://example.com/shared.css") print;');
	expect(css).toContain('@import url("https://example.com/print-only.css") print;');
	expect(css).toContain('@import url("https://example.com/print-screen.css") not all;');
	expect(css).toContain('@import url("https://example.com/screen-same.css") screen;');
	expect(css).not.toContain("screen and screen");
	expect(css).toContain(
		'@import url("https://example.com/feature-screen.css") screen and (min-width: 600px) and (max-width: 1200px);'
	);
	expect(css).toContain(
		'@import url("https://example.com/layered.css") layer(outer.inner) supports((display: grid) and (display: flex));'
	);
});
