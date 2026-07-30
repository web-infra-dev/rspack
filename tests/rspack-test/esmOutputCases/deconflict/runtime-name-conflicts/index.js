import cjsValue from "./require.cjs";

export function context() {}
export const exports = 42;

it("should preserve exported bindings that conflict with runtime names", () => {
	expect(context.name).toBe("context");
	expect(exports).toBe(42);
	expect(cjsValue).toBe(42);
});
