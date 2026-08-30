import "./module";

it("should preserve template coercion and reassigned annotated function effects", () => {
	expect(globalThis.__template_coercion_effect__).toBe(1);
	expect(globalThis.__reassigned_pure_function_effect__).toBe(2);
	expect(globalThis.__reassigned_auto_function_effect__).toBe(2);

	delete globalThis.__template_coercion_effect__;
	delete globalThis.__reassigned_pure_function_effect__;
	delete globalThis.__reassigned_auto_function_effect__;
});
