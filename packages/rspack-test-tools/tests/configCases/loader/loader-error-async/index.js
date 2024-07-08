it("should report error (async)", () => {
	let errored = false;
	try {
		require("./lib?async");
	} catch (e) {
		errored = true;
		expect(e.message).toContain("Failed to load (async)");
	}
	expect(errored).toBeTruthy();
});

it("should report error (callback)", () => {
	let errored = false;
	try {
		require("./lib?callback");
	} catch (e) {
		errored = true;
		expect(e.message).toContain("Failed to load (callback)");
	}
	expect(errored).toBeTruthy();
});
