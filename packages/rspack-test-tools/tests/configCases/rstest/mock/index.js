const TOP_OF_FILE = 1

import { foo, bar } from './barrel'

rs.mock('./foo', () => {
  return { value: 'mockedFoo' }
})

rs.mock('./bar', () => {
  return { value: 'mockedBar' }
})


it('should mock modules', () => {
	expect(foo).toBe('mockedFoo')
	expect(bar).toBe('mockedBar')
})
