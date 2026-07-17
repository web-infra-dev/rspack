it("should leave exact typeof import.meta.env to DefinePlugin", () => {
	expect(typeof import.meta.env).toBe("custom");
});
