import { value } from "./test";

it("should restore lazy reexports across HMR and sideEffects changes", async () => {
	expect(globalThis.__lazyBarrelCompilationState).toBe(
		COMPILER_INDEX === 3 ? 1 : undefined
	);
	delete globalThis.__lazyBarrelCompilationState;

	if (COMPILER_INDEX < 2) {
		expect(value).toBe("a");
		await NEXT_HMR();
		expect(value).toBe("b");
	} else if (COMPILER_INDEX === 2) {
		expect(value).toBe("b");
		await NEXT_HMR();
		expect(value).toBe("a");
	} else {
		expect(value).toBe("a");
	}
	if (COMPILER_INDEX < 4) await NEXT_START();
});

module.hot.accept("./test");
