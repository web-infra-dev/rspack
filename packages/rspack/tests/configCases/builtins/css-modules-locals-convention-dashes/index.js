it("css modules localsConvention with dashes", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		"btn--info_is-disabled_1": "-index-css__btn--info_is-disabled_1 ",
		"btn-info-is-disabled": "-index-css__btn-info_is-disabled ",
		"btn-info-is-disabled-1": "-index-css__btn--info_is-disabled_1 ",
		"btn-info_is-disabled": "-index-css__btn-info_is-disabled ",
		"foo-bar": "-index-css__foo_bar ",
		foo_bar: "-index-css__foo_bar ",
		simple: "-index-css__simple "
	});
});
