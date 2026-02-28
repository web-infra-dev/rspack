it("should not have access to require, module and define", () => {
	expect(typeof define).toBe("undefined");
});
