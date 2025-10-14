import v1 from './vendor1'
import v2 from './vendor2'

import('./async')

it('should not contain vendor1 and vendor2 in current chunk', () => {
	expect(v1).toBe('vendor1')
	expect(v2).toBe('vendor2')
})
