import "./style.css";

it("should support rspackIgnore as an alias for webpackIgnore", () => {
	const links = document.getElementsByTagName("link");
	const css = links[1].sheet.css;
	expect(css).toContain('@import /* rspackIgnore: true */ url("./missing.css")');
	expect(css).toContain('/* rspackIgnore: true */ url("./missing.png")');
});
