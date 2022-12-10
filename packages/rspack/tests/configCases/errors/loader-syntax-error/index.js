it("should report syntax error", () => {
  let errored = false;
	try {
		require("./lib");
	} catch (e) {
    errored = true;
		expect(e.message).toContain("SyntaxError");
	}
  expect(errored).toBeTruthy()
});
