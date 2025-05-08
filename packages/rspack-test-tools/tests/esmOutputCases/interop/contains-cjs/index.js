import { foo } from './foo.cjs'
import v1 from './foo.cjs'
import v2 from './from-esm.cjs'
import { value } from './foo.js'

it('should have correct import', () => {
	expect(v1).toHaveProperty('foo')
	expect(v1.foo).toBe(42)

	expect(v2).toBe('esm')

	expect(foo).toBe(42)
	expect(value).toBe('value')
})
