const { foo: hoistedFoo } = require('./src/barrel')

rs.mockRequire('./src/foo')

it('unmockRequire should be hoisted', () => {
	expect(hoistedFoo).toBe('foo')
	rs.unmockRequire('./src/foo')
	expect(require('./src/foo').value).toBe('foo')
})

it('doUnmockRequire should work', () => {
	rs.doMockRequire('./src/foo')
	expect(require('./src/foo').value).toBe('mocked_foo')

	rs.doUnmockRequire('./src/foo')
	expect(require('./src/foo').value).toBe('foo')
})
