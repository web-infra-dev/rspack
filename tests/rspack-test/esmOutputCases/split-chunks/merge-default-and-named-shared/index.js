import { sharedDefault } from "./a";
import { answer } from "./b";

it("should forward default and named exports from an auto-extracted shared chunk", async () => {
	const mod = await import(/* webpackIgnore: true */ "./main.mjs");

	expect(mod.sharedDefault()).toBe("shared-default");
	expect(mod.answer).toBe(42);
});

export { answer, sharedDefault };
