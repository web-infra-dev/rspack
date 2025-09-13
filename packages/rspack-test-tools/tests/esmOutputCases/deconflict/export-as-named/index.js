import { value } from './foo'

export { value as v }

it('should have deconflicted symbol', async () => {
	let { v } = await import(/* webpackIgnore: true */ './main.mjs');
	expect(v).toBe(42)
})
