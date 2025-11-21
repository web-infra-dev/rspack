it("should load esm loaders", () => {
	expect(require("!!esm-loader!")).toBe("foo")
})
