import { foo } from '../src/barrel'

rstest.mock('../src/foo')

const getGlobalActual = () => rstest.importActual('../src/foo');

afterEach(() => {
	rstest.doUnmock('../src/foo')
})

it('importActual from global scope should work', async () => {
	expect(foo).toBe('mocked_foo')
	const originalFoo = await rstest.importActual('../src/foo')
	expect(originalFoo.value).toBe('foo')
	expect((await getGlobalActual()).value).toBe('foo')
})
