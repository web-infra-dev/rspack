import value from './esm-for-concate'

it('should check stats', () => {
	expect(value).toBe(42)
	const module = __STATS__.modules.find(m => {
		return m.identifier.replaceAll('\\', '/').includes('configCases/concatenate-modules/stats-orphan/esm-for-concate.js')
	})
	expect(module.orphan).toBe(true)
})
