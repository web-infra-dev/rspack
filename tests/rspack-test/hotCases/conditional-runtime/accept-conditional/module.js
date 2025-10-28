import { f } from "./shared";

export async function test(next) {
	expect(f()).toBe(42);
	await next();
	expect(f()).toBe(43);
}
