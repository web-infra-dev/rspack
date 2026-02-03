import { foo } from '../src/barrel'
import { rs } from '@rstest/core';

rstest.mock('../src/foo')

const getGlobalActual = () => rstest.importActual('../src/foo');

it('importActual from global scope should not works when globals is false', async () => {
	expect(foo).toBe('foo')
	const originalFoo = await rstest.importActual('../src/foo')
	expect(originalFoo.value).toBeUndefined()
	expect((await getGlobalActual()).value).toBeUndefined()
})

it('importActual from esm import should works when globals is false', async () => {
	const originalFoo = await rs.importActual('../src/foo')
	expect(originalFoo.value).toBe('foo')
})
