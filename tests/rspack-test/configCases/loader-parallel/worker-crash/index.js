it("should turn a worker crash into a module build error", () => {
	expect(() => require("./resource")).toThrow("Module build failed");
});
