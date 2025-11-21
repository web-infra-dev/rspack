it("build error module should have 'throw error'", () => {
	expect(() => require("./index.scss")).toThrow("Module build failed");
});
