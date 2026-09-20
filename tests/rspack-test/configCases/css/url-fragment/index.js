import "./style.css";
import "./escaped.css";

it("should preserve same-document URLs without creating dependencies", () => {
	const css = getLinkSheet(document.querySelector("link"));

	// Cases copied from webpack's test/configCases/css/url/style.css.
	expect(css).toContain("a18: url(#highlight);");
	expect(css).toContain("a19: url('#line-marker');");
	expect(css).toContain('clip-path: url("#clip");');
	expect(css).toContain("filter: url( #filter );");
	expect(css).toContain("mask: url('#');");
	expect(css).toContain(String.raw`fill: url(\#escaped);`);
	expect(css).toContain(String.raw`stroke: url('\23 escaped-hex');`);
});

it("should still resolve asset URLs with fragments", () => {
	const css = getLinkSheet(document.querySelector("link"));

	expect(css.match(/filter: url\("filters\.svg#myFilter"\)/g)).toHaveLength(2);
});
