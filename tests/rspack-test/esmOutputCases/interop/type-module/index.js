export { foo } from './cjs.cjs'
export { bar } from './cjs-unknown.cjs'
export { default } from './cjs-unknown.cjs'

it('should have correct re-export', async () => {
	const { foo, bar, default: defaultV } = await import(/*webpackIgnore: true*/ './main.mjs')
	expect(foo()).toBe('foo')
	expect(bar()).toBe('bar')
	expect(defaultV.bar()).toBe('bar')
})
