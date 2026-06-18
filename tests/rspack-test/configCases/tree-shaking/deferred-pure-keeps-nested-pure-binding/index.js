// `sideEffectOnly` is intentionally NOT imported — its value is unused.
import { keep } from "./mod";

it("keeps a nested pure binding consumed by a retained deferred-impure expression", () => {
	expect(keep).toBe("kept");

	// `register(config)` was retained (register is impure) and ran for its side
	// effect, recording `config`. `config` must be the real { id: "CFG" } — if the
	// transitive propagation severed it (because `sideEffectOnly`'s value is
	// unused) or dropped `register`'s impure check, this would be `undefined`.
	expect(globalThis.__deferredPureNestedBindingRegistered).toEqual({ id: "CFG" });
});
