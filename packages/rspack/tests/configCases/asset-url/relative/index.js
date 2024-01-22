it("should compile", () => {
	const url = new URL("./index.css?query=yes#fragment", import.meta.url).href;
	expect(url).toBeDefined();
});
