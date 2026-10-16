const state = require("./state");

it("modules re-instantiated during dispose must not stay active", async () => {
	await NEXT_HMR();

	// m1 and m2 were removed by the update. Their dispose handlers
	// re-required each other, so the module that had already been removed
	// from the cache got re-instantiated during dispose.
	expect(state.actives.length).toBe(2);
	expect(state.actives[0]()).toBe(false);
	expect(state.actives[1]()).toBe(false);
});

require("./a");
module.hot.accept("./a");
