it("should compile if loader map returns an empty string (#3418)", () => {
	const actual = require("./lib");
	expect(actual).toBe("lib-foo");
});
