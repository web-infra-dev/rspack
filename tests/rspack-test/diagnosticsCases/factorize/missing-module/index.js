it("should throw if the module is missing", () => {
	let errored = false;
	try {
		require("./missing-module");
	} catch (err) {
		errored = true;
		expect(err.message).toContain("Cannot find module './missing-module'");
	}
	expect(errored).toBeTruthy();
});
