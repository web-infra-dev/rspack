import value from "./file";
import { stable } from "./stable";

it("should reduce affected modules in module codegen on persistent cache restart", async () => {
	const asyncMod = await import("./async-module");

	expect(value).toBe(1);
	expect(stable).toBe("stable");
	expect(asyncMod.value).toBe(42);

	if (COMPILER_INDEX == 0) {
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(1);
		await NEXT_HMR();
		expect(value).toBe(2);
	}
});

module.hot.accept("./file");
