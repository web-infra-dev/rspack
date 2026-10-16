const state = require("./state");

it("modules re-instantiated during dispose must be disposed", async () => {
	await NEXT_HMR();

	// m1 and m2 were removed by the update. Their dispose handlers
	// re-required each other, so instances created after their id had
	// already been disposed were re-instantiated during the dispose phase.
	expect(state.actives.length).toBeGreaterThanOrEqual(2);
	for (let i = 0; i < state.actives.length; i++) {
		expect(state.actives[i]()).toBe(false);
	}

	// Re-created instances are disposed through the same path, so their
	// dispose handlers run once more during the cleanup pass.
	for (const moduleId of ["./m1.js", "./m2.js"]) {
		expect(state.disposeRuns.filter((id) => id === moduleId).length).toBe(2);
	}
});

require("./a");
module.hot.accept("./a");
