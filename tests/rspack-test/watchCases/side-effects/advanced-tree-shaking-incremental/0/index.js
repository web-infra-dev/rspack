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

	if (WATCH_STEP === "1") {
		expect(orphanModules).not.toContain("./bridge.js");
	} else {
		expect(orphanModules).toContain("./bridge.js");
	}
});
