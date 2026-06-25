export * from './foo'

it('should not collide inline re-export with live local binding', async () => {
	const mod = await import(/* webpackIgnore: true */ './main.mjs');
	expect(mod.foo).toBe('foo');
	expect(globalThis.__inlineExportLocalConflict).toBe('foo');
})
