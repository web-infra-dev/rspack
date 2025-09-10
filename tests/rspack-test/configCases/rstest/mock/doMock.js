const TOP_OF_FILE = 1


it('doMock should work', async () => {

	rs.doMock('./src/foo', () => {
		return { value: 'mockedFoo2' }
	})
	const { foo } = await import('./src/barrel')
	expect(foo).toBe('mockedFoo2')
})
