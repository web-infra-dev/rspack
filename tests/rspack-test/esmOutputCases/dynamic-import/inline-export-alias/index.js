it('should preserve aliases for reused inline export bindings', async () => {
	const mod = await import('./foo');

	expect(mod.a).toBe(1);
	expect(mod.b).toBe(1);
	expect(globalThis.__inlineExportAlias).toBe(1);
});
