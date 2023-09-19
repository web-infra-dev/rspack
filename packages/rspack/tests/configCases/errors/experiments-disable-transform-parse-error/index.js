
it("should not be able to parse decorator if `disableTransformByDefault` is enabled", () => {
	let error = null;
	try {
		require("./foo.js")
	} catch (e) {
		error = e
	}
	expect(error).toBeTruthy()
	expect(error.message).toContain("Expression expected")
});
