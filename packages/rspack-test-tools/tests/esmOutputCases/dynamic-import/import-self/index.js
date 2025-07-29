import './value.js'

const conflit = 42;

it('should have access to the value from the same file', async () => {
	const { conflit: c } = await import('./value.js')

	expect(conflit).toBe(42)
	expect(c).toBe(24)
})
