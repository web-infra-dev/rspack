it("should load chunk with patched chunk handler", () => {
	return import("./App").then(({ default: App }) => {
		const rendered = App();
		console.log(rendered)
		expect(rendered).toBe(
			"App fetched with Chunk Handler PASS"
		);
	});
});
