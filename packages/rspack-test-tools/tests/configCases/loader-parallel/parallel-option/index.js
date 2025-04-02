it("should run loader in parallel", () => {
	expect(require("./lib.js")).toBe(SUPPORTS_IMPORT_FN ? true : undefined)
})
