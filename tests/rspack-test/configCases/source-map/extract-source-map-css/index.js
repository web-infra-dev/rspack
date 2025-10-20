import "./app.css";

it("should compile", () => {
	const links = Array.from(document.getElementsByTagName("link"));

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		const css = getLinkSheet(link);
		expect(css).toContain(".row");
		expect(css).toContain(".col-inner");
		expect(css).toContain("/*# sourceMappingURL=bundle0.css.map*/");
	}
});
