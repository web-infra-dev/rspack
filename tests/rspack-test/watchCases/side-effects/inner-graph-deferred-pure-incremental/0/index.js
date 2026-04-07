import { used } from "./consumer";

it("should refresh deferred pure checks for unchanged importers", () => {
	expect(used).toBe("ok");
	expect(globalThis.__innerGraphTracker ?? []).toEqual(
		WATCH_STEP === "1" ? ["impure"] : []
	);
});
