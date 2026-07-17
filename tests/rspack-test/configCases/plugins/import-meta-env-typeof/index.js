it("should handle exact typeof import.meta.env in ImportMetaPlugin", () => {
	expect(typeof import.meta.env).toBe("custom");
});
