it("should render only currently referenced lazy reexports", () => {
	const namespaceReferenced = ["1", "3", "5", "8"].includes(WATCH_STEP);
	expect(BARREL_FACTORIES[0].includes("createStore:")).toBe(namespaceReferenced);
	expect(BARREL_FACTORIES[1].includes("makeStore:")).toBe(namespaceReferenced);
	if (WATCH_STEP === "0" || WATCH_STEP === "1") {
		STATE[`${CONFIG_KEY}-${namespaceReferenced}`] = BARREL_FACTORIES;
	} else if (Number(WATCH_STEP) < 7) {
		expect(BARREL_FACTORIES).toEqual(STATE[`${CONFIG_KEY}-${namespaceReferenced}`]);
	}
});
