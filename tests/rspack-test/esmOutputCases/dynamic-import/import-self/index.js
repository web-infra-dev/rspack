import './value.js'

const conflict = 42;

it('should have access to the value from the same file', async () => {
	const { conflict: c } = await import('./value.js')

	expect(conflict).toBe(42)
	expect(c).toBe(24)
})
