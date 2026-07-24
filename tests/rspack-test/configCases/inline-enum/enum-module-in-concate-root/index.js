import { INLINE } from "./lib.ts";

it("should work", () => {
	expect(INLINE.Foo).toBe(0);
});

import "./foo?a";

import("./module.js");

it("should not emit a getter for an enum whose properties are inlined", async () => {
	const { getGeneratedSource } = await import("./module.js");
	const generated = getGeneratedSource();
	expect(generated.includes("/* binding */ lib_INLINE)")).toBe(false);
	expect(generated.includes("// INLINED EXPORTS: INLINE")).toBe(true);
});
