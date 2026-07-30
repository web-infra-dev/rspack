export function rspackRequire() {}
export function publicPath() {}
export function modules() {}
export function context() {}
export const exports = 42;

export const actualNames = [
	rspackRequire.name,
	publicPath.name,
	modules.name,
	context.name,
];

it("should not reserve runtime names when no runtime is emitted", () => {
	expect(actualNames).toEqual([
		"rspackRequire",
		"publicPath",
		"modules",
		"context",
	]);
	expect(exports).toBe(42);
	expect(value).toBe(42);
});
import { value } from "./value.js";
