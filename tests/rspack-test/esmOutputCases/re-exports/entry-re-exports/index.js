export * from './foo'

it('should have correct output for entry re-exports', async () => {
	const { foo, bar } = await import(/* webpackIgnore: true */ './main.mjs');
	expect(foo).toBe(1);
	expect(bar).toBe(2);
})
