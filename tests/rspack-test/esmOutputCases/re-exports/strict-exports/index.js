import {foo} from './cjs'

it('should only contains needed exports', async () => {
	await import('./chunk')
	expect(foo()).toBe(42)

	const exports = await import(/*webpackIgnore: true*/'./main.mjs')

	expect(Reflect.ownKeys(exports).filter(t => typeof t === 'string').length).toBe(1)
	const {value} = exports

	expect(value()()).toBe(42)
})

export const value = () => foo
