import { keep } from "./mod";

it("drops an unused pure deferred call but keeps an unused impure one", () => {
	expect(keep).toBe("kept");

	// The impure deferred call must have executed exactly once.
	expect(globalThis.__keepAndDropImpureCalls).toBe(1);

	// The pure helper module must have been tree-shaken away entirely: only
	// index, mod and impure-lib remain (pure-lib is gone).
	expect(Reflect.ownKeys(__webpack_modules__).length).toBe(3);
});
