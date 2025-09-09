it("basic", () => {
	const css = require("./index.scss");
	expect(css).toEqual(nsObj({}));
});
