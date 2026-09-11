import "./style.css";
import "./pure.module.css";

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


it("should preserve comment token ranges through CSS scanning paths", () => {
	const css = document.getElementsByTagName("link")[1].sheet.css;
	for (const name of [
		"fast-forward",
		"comment-opener",
		"image-set",
		"inner",
		"first",
		"pure",
		"pure-fast-forward",
		"special"
	]) {
		expect(css).toContain(`./missing-${name}.png`);
	}
});


it("should resolve ordinary comments and comments separated from a URL by a token", () => {
	const css = document.getElementsByTagName("link")[1].sheet.css;
	expect(css).not.toContain("../url/img.png");
	expect(css.match(/resolved-img\.png/g)).toHaveLength(2);
});
