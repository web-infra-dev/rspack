
it("should not be able to parse and transform typescript modules if `disableTransformByDefault` is enabled", () => {
	let error = null;
	try {
		require("./ts")
	} catch (e) {
		error = e
	}
	expect(error).toBeTruthy()
	expect(error.message).toContain("Expression expected")
});

it("should not be able to parse and transform tsx modules if `disableTransformByDefault` is enabled", () => {
	let error = null;
	try {
		require("./tsx")
	} catch (e) {
		error = e
	}
	expect(error).toBeTruthy()
	expect(error.message).toContain("Expected '(', got '<'")
});

it("should not be able to parse and transform jsx modules if `disableTransformByDefault` is enabled", () => {
	let error = null;
	try {
		require("./jsx")
	} catch (e) {
		error = e
	}
	expect(error).toBeTruthy()
	expect(error.message).toContain("Expression expected")
});
