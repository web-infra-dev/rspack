it("should include loader thrown error", () => {
  let errored = false;
	try {
		require("./lib");
	} catch (e) {
    errored = true;
		expect(e.message).toContain("Failed to load");
	}
  expect(errored).toBeTruthy()
});
