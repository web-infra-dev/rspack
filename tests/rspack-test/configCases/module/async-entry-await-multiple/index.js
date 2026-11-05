import state from "./state.js";

// The harness runs these once the bundle settled, so they must be registered before
// the `await`: an `it` after it would never run in a bundle that settles too early.
it("should await an async entry that is not the last one", () => {
	expect(state.first).toBe("first");
});

it("should await every async entry, not only one", () => {
	expect(state.second).toBe("second");
});

state.second = await new Promise((resolve) => {
	setTimeout(() => resolve("second"), 50);
});
