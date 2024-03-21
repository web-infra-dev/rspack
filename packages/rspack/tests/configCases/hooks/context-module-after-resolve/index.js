it("should compile", () => {
	try {
		["foo.js"].map(file => {
			require("./dir/" + file);
		});
	} catch (e) {
		expect(e.message).toContain("Cannot find module './dir'")
	}

	expect.assertions(1)
});
