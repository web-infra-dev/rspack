const { foo } = require('./src/barrel');

rs.mockRequire('./src/foo')

const getActual = () => rs.requireActual('./src/foo');

it('requireActual should works', async () => {
	expect(foo).toBe('mocked_foo')
	const originalFoo = rs.requireActual('./src/foo')
	expect(originalFoo.value).toBe('foo')
	expect(getActual().value).toBe('foo')
})
