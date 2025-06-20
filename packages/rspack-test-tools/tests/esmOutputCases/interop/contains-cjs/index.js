import { foo } from './foo.cjs'
import { value } from './foo.js'

it('should have correct import', () => {
	expect(foo).toBe(42)
	expect(value).toBe('value')
})
