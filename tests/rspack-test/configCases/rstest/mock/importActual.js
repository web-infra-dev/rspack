import { foo } from './src/barrel'
import { rstest } from '@rstest/core';
rstest.mock('./src/foo')

const getActual = () => rstest.importActual('./src/foo');
const getGlobalActual = () => rstest.importActual('./src/foo');

it('importActual should works', async () => {
	expect(foo).toBe('mocked_foo')
	const originalFoo = await rstest.importActual('./src/foo')
	expect(originalFoo.value).toBe('foo')
	expect((await getActual()).value).toBe('foo')
	expect((await getGlobalActual()).value).toBe('foo')
})
