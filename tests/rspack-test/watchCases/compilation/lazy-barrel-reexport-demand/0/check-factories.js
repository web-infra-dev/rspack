it("should preserve rendered getters when the lazy barrel is reused", () => {
	// A full make creates new dependencies; a reused lazy graph only adds getters.
	const namespaceReferenced = ["1", "3", "5", "8"].includes(WATCH_STEP);
	const namespaceRendered = namespaceReferenced ||
		(CONFIG_INCREMENTAL && Number(WATCH_STEP) > 0 && Number(WATCH_STEP) < 7);
	expect(BARREL_FACTORIES[0].includes("createStore:")).toBe(namespaceRendered);
	expect(BARREL_FACTORIES[1].includes("makeStore:")).toBe(namespaceRendered);
	if (WATCH_STEP === "0" || WATCH_STEP === "1") {
		STATE[CONFIG_KEY] = BARREL_FACTORIES;
	} else if (CONFIG_INCREMENTAL && Number(WATCH_STEP) < 7) {
		expect(BARREL_FACTORIES).toEqual(STATE[CONFIG_KEY]);
	}
});
