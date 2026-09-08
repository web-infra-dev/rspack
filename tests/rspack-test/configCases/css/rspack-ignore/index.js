import "./style.css";

it("should support rspackIgnore as an alias for webpackIgnore", () => {
	const links = document.getElementsByTagName("link");
	const css = links[1].sheet.css;
	expect(css).toContain('@import /* rspackIgnore: true */ url("./missing.css")');
	expect(css).toContain(
		'@import /* webpackIgnore : true */ url("./missing-whitespace.css")'
	);
	expect(css).toContain(
		'/* webpackIgnore: true, webpackChunkName: "ignored" */ url("./missing-webpack-mixed.css")'
	);
	expect(css).toContain(
		'/* webpackChunkName: "ignored", rspackIgnore: true */ url("./missing-rspack-mixed.css")'
	);
	expect(css).toContain('/* rspackIgnore: true */ url("./missing.png")');
});
