import { value } from "./test";

it("should restore lazy reexports across HMR and sideEffects changes", async () => {
	expect(globalThis.__lazyBarrelCompilationState).toBe(
		COMPILER_INDEX === 3 ? 1 : undefined
	);
	delete globalThis.__lazyBarrelCompilationState;

	if (COMPILER_INDEX === 0) {
		expect(value).toBe("a");
		await NEXT_HMR();
		expect(value).toBe("b");
		await NEXT_START();
	}
	if (COMPILER_INDEX === 1) {
		expect(value).toBe("a");
		await NEXT_HMR();
		expect(value).toBe("b");
		await NEXT_START();
	}
	if (COMPILER_INDEX === 2) {
		expect(value).toBe("b");
		await NEXT_HMR();
		expect(value).toBe("a");
		await NEXT_START();
	}
	if (COMPILER_INDEX >= 3) {
		expect(value).toBe("a");
		if (COMPILER_INDEX === 3) await NEXT_START();
	}
});

module.hot.accept("./test");
