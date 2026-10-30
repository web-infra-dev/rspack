import value from "./dep";

// `extra` is provided by the loader-injected import after the finishModules
// rebuild, so the bundle always executes the rewritten entry.
it("should add a dependency during rebuildModule in finishModules", function() {
	switch (WATCH_STEP) {
		case "0":
			expect(value).toBe("initial");
			break;
		default:
			expect(value).toBe("changed");
			break;
	}
	expect(extra).toBe("extra");
});
