import { value } from "./dep.js";

it("should preserve frozen build metadata across restarts and replace it on recovery", async () => {
	expect(value).toBe(COMPILER_INDEX < 2 ? "initial" : "recovered");
	if (COMPILER_INDEX === 1) await NEXT_START();
});
