import { foo } from './src/barrel'
import { rs } from '@rstest/core';
rstest.mock('./src/foo')

const getActual = () => rs.importActual('./src/foo');
const getGlobalActual = () => rstest.importActual('./src/foo');

it('importActual should works', async () => {
	expect(foo).toBe('mocked_foo')
	expect((await getActual()).value).toBe('foo')
	const originalFoo = await rstest.importActual('./src/foo')
	expect(originalFoo.value).toBe('foo')
	expect((await getGlobalActual()).value).toBe('foo')
})
