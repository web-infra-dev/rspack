it("should update getters when namespace demand is added and removed", () => {
	const namespaceReferenced = WATCH_STEP === "1" || WATCH_STEP === "3";
	expect(BARREL_FACTORIES[0].includes("createStore:")).toBe(namespaceReferenced);
	expect(BARREL_FACTORIES[1].includes("makeStore:")).toBe(namespaceReferenced);
	if (WATCH_STEP === "0") {
		STATE[CONFIG_KEY] = BARREL_FACTORIES;
	} else if (WATCH_STEP === "2") {
		expect(BARREL_FACTORIES).toEqual(STATE[CONFIG_KEY]);
	}
});
