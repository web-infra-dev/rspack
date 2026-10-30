import value from "./dep";

it("should survive incremental rebuilds after rebuildModule in finishModules", function() {
	switch (WATCH_STEP) {
		case "0":
			expect(value).toBe("initial");
			break;
		default:
			expect(value).toBe("changed");
			break;
	}
});
