import { fill } from 'lodash-es'
import { foo } from './src/barrel'

rs.mock('./src/foo')
rs.mock('lodash-es')

it('should mock to __mocks__', () => {
	expect(foo).toBe('mocked_foo')
	expect(fill).toBe('mocked_lodash_fill')
})
