it("should run loader in parallel", () => {
	expect(require("./loader.js!./lib.js"), true)
})
