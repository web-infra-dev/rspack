import { b } from './shared.js'

it('should run well', async () => {
	expect(b).toBe(2)
	const d = await import("./d").then(m => m.d);
	expect(d).toBe(4);
})
