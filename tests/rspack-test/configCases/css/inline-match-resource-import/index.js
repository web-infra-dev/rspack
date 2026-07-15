import "./App.vue.css!=!./pitcher.js!./App.vue";

it("should include CSS imported through an inline match resource", () => {
	const css = getLinkSheet(document.querySelector("link"));

	expect(css).toContain(".hello");
	expect(css).toContain("143px");
	expect(css).toContain("rebeccapurple");
});
