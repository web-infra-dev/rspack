import "./app.css";

it("should compile", () => {
	const links = document.getElementsByTagName("link");

	// Skip first because import it by default
	for (const link of links.slice(1)) {
		expect(link.sheet.css).toContain(".row");
		expect(link.sheet.css).toContain(".col-inner");
		expect(link.sheet.css).toContain("/*# sourceMappingURL=bundle0.css.map*/");
	}
});
