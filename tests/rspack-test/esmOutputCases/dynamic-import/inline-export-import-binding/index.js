it('should preserve aliases for imported inline export bindings', async () => {
	const mod = await import('./foo');

	expect(mod.a).toBe(1);
	expect(mod.b).toBe(1);
	expect(globalThis.__inlineExportImportBinding).toBe(1);
});
