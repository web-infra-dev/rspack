it("should compile a fixture before afterExecute removes it", () => {
	expect(require("temporary-fixture")).toBe("temporary fixture");
});
