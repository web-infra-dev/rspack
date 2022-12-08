it("resolve to internal module shoudl work", () => {
	const css = require("./index.less");
	expect(css).toEqual({});
});
