import {ns} from './bar'
let foo = 234

it('should compile', () => {
	expect(ns).toHaveProperty('foo')
	expect(ns.foo).toBe(123)
	expect(foo).toBe(234)
})

