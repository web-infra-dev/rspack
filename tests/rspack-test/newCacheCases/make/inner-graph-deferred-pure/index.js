import { used } from "./consumer";

it("should reevaluate deferred pure checks after cache recovery", async () => {
	expect(used).toBe("ok");
	if (COMPILER_INDEX === 0) await NEXT_START();
});
