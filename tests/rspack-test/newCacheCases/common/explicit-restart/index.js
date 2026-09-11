import value from "./value";
import stable from "./stable";

it("should preserve HMR and explicit restart steps", async () => {
	recordCompiler(COMPILER_INDEX);
	expect(stable).toBe("stable");
	if (COMPILER_INDEX === 0) {
		expect(value).toBe(1);
		await NEXT_HMR();
		expect(value).toBe(2);
		await NEXT_START();
	} else {
		expect(COMPILER_INDEX).toBe(1);
		expect(value).toBe(3);
	}
});

module.hot.accept("./value");
