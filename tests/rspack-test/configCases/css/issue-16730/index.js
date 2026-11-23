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
