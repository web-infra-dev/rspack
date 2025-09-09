import { a } from "./shared.js";

it('should run well', async () => {
	expect(a).toBe(1)
	const c = await import("./c").then(m => m.c);
	expect(c).toBe(3);
})
