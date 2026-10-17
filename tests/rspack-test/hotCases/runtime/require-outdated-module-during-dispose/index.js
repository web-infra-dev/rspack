const state = require("./state");

it("requiring outdated modules while disposing must not re-instantiate them", async () => {
	await NEXT_HMR();

	// m1 and m2 were removed by the update. Their dispose handlers
	// re-required each other; the requires must resolve to the instances
	// that are being disposed instead of running the removed factories again.
	expect(state.actives.length).toBe(2);
	expect(state.actives[0]()).toBe(false);
	expect(state.actives[1]()).toBe(false);
	for (const moduleId of ["./m1.js", "./m2.js"]) {
		expect(state.disposeRuns.filter((id) => id === moduleId).length).toBe(1);
	}
});

require("./a");
module.hot.accept("./a");
