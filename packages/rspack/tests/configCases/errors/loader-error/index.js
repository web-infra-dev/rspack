it("should include loader error", () => {
	try {
		require("./lib");
	} catch (e) {
		expect(e.message).toContain("Failed to load");
	}
});
