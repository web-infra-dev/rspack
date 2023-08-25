it("should use inline builtin loader with ident", () => {
	const { lib } = require("builtin:swc-loader??builtin-swc-loader!./lib.ts");
	// expect(lib).toBe("lib");
});
