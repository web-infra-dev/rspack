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
