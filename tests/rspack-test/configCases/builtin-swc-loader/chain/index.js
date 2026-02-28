it("should support chaining builtin:swc-loader", () => {
	const { lib, lib2 } = require("./lib");
	expect(lib).toBe("lib");
	expect(lib2).toBe("lib2");
});
