import { foo } from './src/barrel'

rstest.mock('./src/foo')

it('importActual should works', async () => {
	expect(foo).toBe('mocked_foo')
	const originalFoo = await rstest.importActual('./src/foo')
	expect(originalFoo.value).toBe('foo')
})
