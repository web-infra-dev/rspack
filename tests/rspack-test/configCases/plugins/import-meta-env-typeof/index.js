it("should honor exact typeof import.meta.env definitions", () => {
	expect(typeof import.meta.env).toBe("custom");
});
