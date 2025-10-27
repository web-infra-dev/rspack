it('should has connection to lib only in closure', () => {
	const { foo } = require('./lib')
	expect(foo()).toBe(42)
})
