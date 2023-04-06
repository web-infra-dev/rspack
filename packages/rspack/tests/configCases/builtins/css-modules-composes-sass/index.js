it("css modules in scss", () => {
	const style = require("./index.scss");
	expect(style).toEqual({
		bar: "-index-scss__bar -foo-scss__foo"
	});
});
