it("should inject an external @import from a lazy-loaded style module", async () => {
	await import("./lazy.js");

	const allCSS = Array.from(window.document.getElementsByTagName("style")).map(
		s => s.textContent
	);

	expect(
		allCSS.some(c => c.includes('@import url("https://example.com/external.css");'))
	).toBe(true);
	expect(allCSS.some(c => c.includes("color: red"))).toBe(true);
});

it("should preserve inherited conditions for external @import rules", async () => {
	await import("./lazy.js");

	const allCSS = Array.from(window.document.getElementsByTagName("style")).map(
		s => s.textContent
	);

	expect(
		allCSS.some(c => c.includes('@import url("https://example.com/print-only.css") print;'))
	).toBe(true);
	expect(allCSS.some(c => c.includes("@media print") && c.includes("color: blue"))).toBe(
		true
	);
});

it("should not deduplicate the same external @import across different conditions", async () => {
	await import("./lazy.js");

	const allCSS = Array.from(window.document.getElementsByTagName("style")).map(
		s => s.textContent
	);

	const plainCount = allCSS.filter(c =>
		c.includes('@import url("https://example.com/external.css");')
	).length;
	const screenCount = allCSS.filter(c =>
		c.includes('@import url("https://example.com/external.css") screen;')
	).length;

	expect(plainCount).toBe(1);
	expect(screenCount).toBe(1);
	expect(allCSS.some(c => c.includes("@media screen") && c.includes("color: green"))).toBe(
		true
	);
});

it("should intersect nested media query syntax properly", async () => {
	await import("./lazy.js");

	const allCSS = Array.from(window.document.getElementsByTagName("style")).map(
		s => s.textContent
	);

	expect(
		allCSS.some(c => c.includes('@import url("https://example.com/screen-same.css") screen;'))
	).toBe(true);
	expect(allCSS.some(c => c.includes("screen and screen"))).toBe(false);

	expect(
		allCSS.some(c =>
			c.includes(
				'@import url("https://example.com/feature-screen.css") screen and (min-width: 600px) and (max-width: 1200px);'
			)
		)
	).toBe(true);

	expect(
		allCSS.some(c => c.includes('@import url("https://example.com/all-screen.css") screen;'))
	).toBe(true);

	expect(
		allCSS.some(c => c.includes('@import url("https://example.com/screen-print.css") not all;'))
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes(
				'@import url("https://example.com/only-screen.css") only screen and (min-width: 600px);'
			)
		)
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes(
				'@import url("https://example.com/or-and.css") ((min-width: 600px) or (orientation: landscape)) and (hover: hover);'
			)
		)
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes(
				'@import url("https://example.com/screen-or.css") screen and ((min-width: 600px) or (orientation: landscape));'
			)
		)
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes(
				'@import url("https://example.com/not-color.css") (min-width: 600px) and (not (color));'
			)
		)
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes('@import url("https://example.com/not-print-screen.css") screen;')
		)
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes('@import url("https://example.com/not-screen-screen.css") not all;')
		)
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes(
				'@import url("https://example.com/not-print-feature.css") screen and (min-width: 600px), speech and (min-width: 600px);'
			)
		)
	).toBe(true);

	expect(
		allCSS.some(c =>
			c.includes(
				'@import url("https://example.com/layered.css") layer(outer.inner) supports((display: grid) and (display: flex));'
			)
		)
	).toBe(true);
});
