import { foo } from '../src/barrel'
import { rs } from '@rstest/core';

try {
	rstest.mock('../src/foo')
} catch {
	// `globals: false` intentionally leaves global rstest APIs untransformed.
	// Newer Rstest versions throw for untransformed mock APIs, so keep this
	// fixture focused on verifying global importActual does not work.
}

const getGlobalActual = async () => {
	try {
		return await rstest.importActual('../src/foo')
	} catch {
		return {}
	}
};

it('importActual from global scope should not work when globals is false', async () => {
	expect(foo).toBe('foo')
	const originalFoo = await getGlobalActual()
	expect(originalFoo.value).toBeUndefined()
	expect((await getGlobalActual()).value).toBeUndefined()
})

it('importActual from esm import should work when globals is false', async () => {
	const originalFoo = await rs.importActual('../src/foo')
	expect(originalFoo.value).toBe('foo')
})
