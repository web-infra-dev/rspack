import "./pkg/side.js";

it("should restore side-effect imports after changing package metadata", () => {
	expect(STATE.sideEffect).toBe(WATCH_STEP === "0" ? undefined : true);
	const entry = __STATS__.modules.find(module => module.name === "./index.js");
	expect(entry.built).toBe(WATCH_STEP === "0");
	const side = __STATS__.modules.find(module => module.name === "./pkg/side.js");
	if (WATCH_STEP === "0") {
		expect(side).toBeUndefined();
	} else {
		expect(side).toMatchObject({ built: true });
	}
});
