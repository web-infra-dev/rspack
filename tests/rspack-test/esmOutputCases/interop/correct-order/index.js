import v1 from './foo.cjs'
import v2 from './bar.cjs'

// the order should be import order not access order
it('should have correct order', () => {
	expect(v2).toBe('bar');
	expect(v1).toBe('foo');
	expect(globalThis.testValue).toBe('bar')
})
