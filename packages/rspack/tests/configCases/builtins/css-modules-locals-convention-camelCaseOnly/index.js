it("css modules localsConvention with camelCaseOnly", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		btnInfoIsDisabled: "-index-css__btn-info_is-disabled ",
		btnInfoIsDisabled1: "-index-css__btn--info_is-disabled_1 ",
		fooBar: "-index-css__foo_bar ",
		simple: "-index-css__simple "
	});
});
