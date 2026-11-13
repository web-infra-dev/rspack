function readCandidate() {
	try {
		return require("./candidate");
	} catch (error) {
		expect(error.code).toBe("MODULE_NOT_FOUND");
		return "missing";
	}
}

it("should reuse missing resolutions and invalidate them when files change", async () => {
	expect(readCandidate()).toBe(
		COMPILER_INDEX === 2 || COMPILER_INDEX === 3 ? "present" : "missing"
	);
	if (COMPILER_INDEX < 5) {
		await NEXT_START();
	}
});
