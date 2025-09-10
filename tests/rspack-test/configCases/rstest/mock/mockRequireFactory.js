const TOP_OF_FILE = 1

const { foo, bar } = require('./src/barrel')

rs.mockeRequire('./src/foo', () => {
  return { value: 'mockedFoo' }
})

rs.mockeRequire('./src/bar', () => {
  return { value: 'mockedBar' }
})


it('should mock modules', () => {
	expect(foo).toBe('mockedFoo')
	expect(bar).toBe('mockedBar')
})
