import * as ns from "./foo"

it('should import namespace', () => {
	expect(ns.foo).toBe(123);
})
