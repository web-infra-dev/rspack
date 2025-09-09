it('doMock should work', async () => {
	rs.doMockRequire('./src/foo', () => {
		return { value: 'mockedFoo2' }
	})
	const { foo } = require('./src/barrel')
	expect(foo).toBe('mockedFoo2')
})
