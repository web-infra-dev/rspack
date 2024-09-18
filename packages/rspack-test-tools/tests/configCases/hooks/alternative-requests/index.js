it("should compile", () => {
	const nameA = "a";
	const a = require("./dir/" + nameA);

	try {
		const nameB = "b";
		expect(a).toBe("a");

		require("./dir/" + nameB);
	} catch (e) {
		expect(e.message).toContain("Cannot find module './b'");
	}

	expect.assertions(2);
});
