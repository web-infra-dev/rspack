it("should assign identifier-safe module ids with compact-hashed", () => {
	const ids = [require("./a"), require("./b"), require("./c")];

	expect(new Set(ids).size).toBe(ids.length);
	for (const id of ids) {
		expect(id).toMatch(/^[A-Za-z][A-Za-z0-9]*$/);
	}
});
