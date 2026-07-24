import { loaders, usedAtInitialization } from "./barrel";

it("should not emit a dynamic import used only by an unused export", () => {
	expect(globalThis.unusedObjectInitializerRuns).toBe(1);
	expect(loaders).toEqual([]);
	expect(usedAtInitialization.loader).toBeInstanceOf(Promise);
});
