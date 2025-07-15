const TOP_OF_FILE = 1

import { foo, bar } from './src/barrel'

rs.mock('./src/foo', () => {
  return { value: 'mockedFoo' }
})

rs.mock('./src/bar', () => {
  return { value: 'mockedBar' }
})


it('should mock modules', () => {
	expect(foo).toBe('mockedFoo')
	expect(bar).toBe('mockedBar')
})
