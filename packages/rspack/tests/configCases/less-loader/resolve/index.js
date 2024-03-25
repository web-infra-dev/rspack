it("resolve to internal module should work", () => {
	const css = require("./index.less");
	expect(css).toEqual({});
});
