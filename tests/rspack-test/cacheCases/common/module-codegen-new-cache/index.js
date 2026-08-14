import value from "./file";
import { stable } from "./stable";

it("should use the new module codegen cache across restarts and rebuilds", async () => {
	const asyncModule = await import("./async-module");

	expect(value).toBe(1);
	expect(stable).toBe("stable");
	expect(asyncModule.value).toBe(42);

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
