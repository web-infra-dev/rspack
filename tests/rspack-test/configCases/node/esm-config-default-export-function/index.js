it("loads the ESM default factory export", () => {
	expect(EXPORT_KIND).toBe("esm-default-factory");
	expect(FACTORY_ARGS).toEqual({
		hasConfig: true,
		hasTestPath: true
	});
});
