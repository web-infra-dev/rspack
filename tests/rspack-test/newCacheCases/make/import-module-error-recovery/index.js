import value from "./module";
// Keep the shared module in the main graph when the loader's module is removed.
import shared from "./shared";

it("should recover when a loader and the main graph share a module", async () => {
	expect(shared).toBe("shared");
	expect(value).toBe(COMPILER_INDEX === 0 ? "initial shared" : "recovered shared");
	if (COMPILER_INDEX === 0) {
		// The syntax error removes the loader module and its importModule entry.
		await NEXT_HMR({ ignoreErrored: true });
		expect(value).toBe("initial shared");
		await NEXT_HMR();
		expect(value).toBe("recovered shared");
		expect(shared).toBe("shared");
	}
});

module.hot.accept("./module");
