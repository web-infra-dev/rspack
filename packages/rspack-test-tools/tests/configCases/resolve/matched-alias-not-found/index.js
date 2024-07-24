it("should error when alias not match", () => {
	try {
		require("m1")
	} catch (e) {
		expect(e.message).toContain("Cannot find module 'm1'");
	}
})
