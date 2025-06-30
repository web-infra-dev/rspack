import { foo } from './src/barrel'

rs.mock('./src/foo')

it('importActual should works', async () => {
	expect(foo).toBe('mocked_foo')
	const originalFoo = await rs.importActual('./src/foo')
	expect(originalFoo.value).toBe('foo')
})
