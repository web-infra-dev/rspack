rs.mockRequire('./src/foo')
rs.mockRequire('./src/bar')
rs.unmockRequire('./src/foo')

const { foo: hoistedFoo, bar: hoistedBar } = require('./src/barrel')

it('unmockRequire should be hoisted', () => {
	expect(hoistedFoo).toBe('foo')
	expect(require('./src/foo').value).toBe('foo')
})

it('unmockRequire should preserve source order with other hoisted requests', () => {
	expect(hoistedBar).toBe('mocked_bar_with_mocked_foo')
})

it('doUnmockRequire should work', () => {
	rs.doMockRequire('./src/foo')
	expect(require('./src/foo').value).toBe('mocked_foo')

	rs.doUnmockRequire('./src/foo')
	expect(require('./src/foo').value).toBe('foo')
})
