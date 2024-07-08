it("should be able to import missing module from block", async () => {
	let errored = false;
	try {
		await import("./non-exist")
	} catch (err) {
    errored = true
    expect(err.message).toContain("Cannot find module './non-exist'")
	}
  expect(errored).toBeTruthy()
});
