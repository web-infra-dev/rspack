import { bar as local } from './bar'
import { foo as bar } from './foo'

// export foo
export * from './foo'

export { local, bar }

it('test exports', async () => {
	const { local, bar, foo } = await import(/* webpackIgnore: true */ './main.mjs')

	expect(local).toBe('bar')
	expect(bar).toBe('foo')
	expect(foo).toBe('foo')
})
