import { ns } from './bar'

it('should import star export', () => {
	expect(ns).toContain('foo1')
	expect(ns).toContain('foo2')
	expect(ns.foo1).toBe(1)
	expect(ns.foo2).toBe(2)
})
