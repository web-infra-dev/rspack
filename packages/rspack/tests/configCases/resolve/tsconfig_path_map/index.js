it("should require real.js", () => {
	const value = require("fake");
	expect(value).toEqual("real");
});
