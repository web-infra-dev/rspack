it("should reject the bundle import and observe every async entry", async () => {
	await expect(import("failure")).rejects.toThrow("first entry rejected");
	// Give the other entry time to reject, so an unobserved rejection fails the test.
	await new Promise(resolve => setTimeout(resolve, 50));
});
