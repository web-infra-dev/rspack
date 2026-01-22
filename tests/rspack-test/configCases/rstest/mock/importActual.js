import { foo } from './src/barrel'

rstest.mock('./src/foo')

const getActual = () => rstest.importActual('./src/foo');

it('importActual should works', async () => {
	expect(foo).toBe('mocked_foo')
	const originalFoo = await rstest.importActual('./src/foo')
	expect(originalFoo.value).toBe('foo')
	expect((await getActual()).value).toBe('foo')
})
