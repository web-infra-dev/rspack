import value from './index.js'

it('should compile', () => {
	expect(value).toBe(JSON.stringify('bar'))
})
