import defaultValue from "./dynamic?1";

it('should support dynamic import', async () => {
	const value = await import('./dynamic')
	const value2 = await import('./dynamic?1')

	expect(value.default).toBe(42)
	expect(defaultValue).toBe(42)
	expect(value2.default).toBe(42)
})
