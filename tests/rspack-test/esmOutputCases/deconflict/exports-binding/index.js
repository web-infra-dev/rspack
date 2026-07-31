export const exports = 42;
export { exports as beforeExport } from "./before-export";
export { exports as wrappedExports } from "./wrapped";

it("should preserve direct and deconflicted exports bindings", async () => {
	expect(exports).toBe(42);
	const namespace = await import(/* webpackIgnore: true */ "./main.mjs");
	expect(namespace.exports).toBe(42);
	expect(namespace.beforeExport).toBe(43);
	expect(namespace.wrappedExports).toBe(44);
});
