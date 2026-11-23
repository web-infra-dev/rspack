import "./entry.css";

const fs = require("fs");
const path = require("path");

const readBundleCss = () =>
	fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");

it("should preserve inherited conditions and avoid deduplicating external @import across conditions in CSS chunks", () => {
	const css = readBundleCss();

	expect(css.split('@import url("https://example.com/shared.css");').length - 1).toBe(1);
	expect(css.split('@import url("https://example.com/shared.css") print;').length - 1).toBe(1);
	expect(css).toContain('@import url("https://example.com/print-only.css") print;');
	expect(
		css
	).toContain(
		'@import url("https://example.com/layered.css") layer(outer.inner) supports((display: grid) and (display: flex));'
	);
});

it("should intersect nested media query syntax properly in CSS chunks", () => {
	const css = readBundleCss();

	expect(css).toContain('@import url("https://example.com/screen-same.css") screen;');
	expect(css).not.toContain("screen and screen");
	expect(css).toContain(
		'@import url("https://example.com/feature-screen.css") screen and (min-width: 600px) and (max-width: 1200px);'
	);
	expect(css).toContain('@import url("https://example.com/all-screen.css") screen;');
	expect(css).toContain('@import url("https://example.com/print-screen.css") not all;');
	expect(css).toContain(
		'@import url("https://example.com/only-screen.css") only screen and (min-width: 600px);'
	);
	expect(css).toContain(
		'@import url("https://example.com/or-and.css") ((min-width: 600px) or (orientation: landscape)) and (hover: hover);'
	);
	expect(css).toContain(
		'@import url("https://example.com/screen-or.css") screen and ((min-width: 600px) or (orientation: landscape));'
	);
	expect(css).toContain(
		'@import url("https://example.com/not-color.css") (min-width: 600px) and (not (color));'
	);
	expect(css).toContain('@import url("https://example.com/not-print-screen.css") screen;');
	expect(css).toContain('@import url("https://example.com/not-screen-screen.css") not all;');
	expect(css).toContain(
		'@import url("https://example.com/not-print-feature.css") screen and (min-width: 600px), speech and (min-width: 600px);'
	);
});

it("should decode CSS escapes in media query identifiers and keywords", () => {
	const css = readBundleCss();

	expect(css).toContain('@import url("https://example.com/escaped-screen.css") screen;');
	expect(css).toContain(
		'@import url("https://example.com/escaped-and.css") screen and (max-width: 1200px);'
	);
	expect(css).toContain(
		'@import url("https://example.com/escaped-only.css") only screen and (min-width: 600px);'
	);
	expect(css).toContain('@import url("https://example.com/escaped-not-print.css") screen;');
	expect(css).toContain(
		'@import url("https://example.com/escaped-or.css") ((min-width: 600px) \\6f r (orientation: landscape)) and (hover: hover);'
	);
	expect(css).toContain(
		'@import url("https://example.com/escaped-not-cond.css") (min-width: 600px) and (\\6e ot (color));'
	);
	expect(css).toContain('@import url("https://example.com/escaped-nonplain.css") \\31 custom;');
});

it("should preserve inherited conditions and avoid deduplicating external @import with exportType style", async () => {
	await import("./style-entry.style.css");

	const allCSS = Array.from(window.document.getElementsByTagName("style")).map(
		s => s.textContent
	);

	expect(
		allCSS.filter(c => c.includes('@import url("https://example.com/style-shared.css");')).length
	).toBe(1);
	expect(
		allCSS.filter(c => c.includes('@import url("https://example.com/style-shared.css") print;'))
			.length
	).toBe(1);
	expect(
		allCSS.some(c => c.includes('@import url("https://example.com/style-print-only.css") print;'))
	).toBe(true);
	expect(allCSS.some(c => c.includes("@media print") && c.includes("color: blue"))).toBe(true);
});
