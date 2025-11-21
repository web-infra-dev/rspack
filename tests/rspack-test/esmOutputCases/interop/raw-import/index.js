import './bar.js'
import './foo.js'

it('should have correct import', () => {
	expect(globalThis.bar).toBe('bar');
	expect(globalThis.foo).toBe('foo');
})
