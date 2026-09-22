it("should able to run pitching loader that is defined on JS side", () => {
	const { lib, lib2 } = require("./pitching-loader.mjs!./lib");
	expect(lib).toBe("lib");
	expect(lib2).toBe("lib2");
});
