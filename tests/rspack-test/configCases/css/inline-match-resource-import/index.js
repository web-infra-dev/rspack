import "./App.vue.css!=!./pitcher.js!./App.vue";
import "./Charset.vue.css!=!./charset-pitcher.js!./Charset.vue";

it("should include CSS imported through an inline match resource", () => {
	const css = getLinkSheet(document.querySelector("link"));

	expect(css).toContain(".hello");
	expect(css).toContain("143px");
	expect(css).toContain("rebeccapurple");
});

it("should preserve charset from a deduplicated inline match resource import", () => {
	const css = getLinkSheet(document.querySelector("link"));

	expect(css.match(/\.charset-carrier/g)).toEqual([".charset-carrier"]);
	expect(css.match(/@charset/g)).toEqual(["@charset"]);
	expect(css.startsWith('@charset "UTF-8";\n')).toBe(true);
});
