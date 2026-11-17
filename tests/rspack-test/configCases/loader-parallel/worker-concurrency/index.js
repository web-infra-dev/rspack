it("should execute independent modules on native workers", () => {
	const threadIds = [
		require("./m0"), require("./m1"), require("./m2"), require("./m3"),
		require("./m4"), require("./m5"), require("./m6"), require("./m7")
	];
	expect(threadIds.every(id => id > 0)).toBe(true);
});
