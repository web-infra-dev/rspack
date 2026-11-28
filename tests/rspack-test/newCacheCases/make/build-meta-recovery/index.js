import { value } from "./dep.js";

it("should preserve frozen build metadata through failed rebuilds and replace it on recovery", async () => {
	expect(value).toBe(COMPILER_INDEX < 2 ? "initial" : "recovered");
	if (COMPILER_INDEX === 1) {
		await NEXT_HMR({ ignoreErrored: true });
		await NEXT_HMR({ ignoreErrored: true });
		// Becoming synchronous also changes the entry's generated code.
		await NEXT_HMR({ ignoreUnaccepted: true });
		expect(value).toBe("recovered");
		await NEXT_START();
	}
});

module.hot.accept("./dep.js");
