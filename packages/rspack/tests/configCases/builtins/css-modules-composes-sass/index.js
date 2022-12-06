it("css modules in scss", () => {
	const style = require("./index.scss");
	expect(style).toEqual({
		bar: "bar-index.scss foo-foo.scss "
	});
});
