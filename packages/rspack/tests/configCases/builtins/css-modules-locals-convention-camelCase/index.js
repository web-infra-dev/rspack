it("css modules localsConvention with camelCase", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		"btn--info_is-disabled_1": "index-css__btn--info_is-disabled_1",
		"btn-info_is-disabled": "index-css__btn-info_is-disabled",
		btnInfoIsDisabled: "index-css__btn-info_is-disabled",
		btnInfoIsDisabled1: "index-css__btn--info_is-disabled_1",
		fooBar: "index-css__foo_bar",
		foo_bar: "index-css__foo_bar",
		simple: "index-css__simple"
	});
});
