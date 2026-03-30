import { used } from "./dep";

it("should clear stale sideEffectsFree markers on reparse", () => {
	expect(used).toBe("ok");
	expect(globalThis.__explicitSideEffectsFreeTracker ?? []).toEqual(
		WATCH_STEP === "1" ? ["impure"] : []
	);
});
