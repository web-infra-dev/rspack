it("basic", () => {
	const css = require("./index.less");
	expect(css).toEqual(nsObj({}));
});
