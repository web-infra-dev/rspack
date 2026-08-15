it("keeps the CommonJS function export when it has a default property", () => {
	expect(EXPORT_KIND).toBe("commonjs-factory");
	expect(FACTORY_ARGS).toEqual({
		hasConfig: true,
		hasTestPath: true
	});
});
