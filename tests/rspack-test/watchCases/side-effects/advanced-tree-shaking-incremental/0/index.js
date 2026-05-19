import { value } from "./bridge";
import { tracker } from "./tracker";

it("should invalidate stale side effects optimize artifacts", () => {
	expect(value).toBe("ok");

	if (WATCH_STEP === "1") {
		expect(tracker).toEqual(["impure"]);
	} else {
		expect(tracker).toEqual([]);
	}

	const orphanModules = new Set(
		__STATS__.modules.filter(module => module.orphan).map(module => module.name)
	);
	const bridgeModule = __STATS__.modules.find(module => module.name === "./bridge.js");
	const bridgeBailouts = bridgeModule?.optimizationBailout ?? [];

	if (WATCH_STEP === "1") {
		expect(orphanModules).not.toContain("./bridge.js");
		expect(bridgeBailouts).toContain("Call with side effects");
	} else {
		expect(orphanModules).toContain("./bridge.js");
		expect(bridgeBailouts).not.toContain("Call with side effects");
	}
});
