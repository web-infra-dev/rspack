import { used } from "./consumer";

console.log(used);

it("should reevaluate deferred pure checks after cache recovery", async () => {
	if (COMPILER_INDEX === 0) await NEXT_START();
});
