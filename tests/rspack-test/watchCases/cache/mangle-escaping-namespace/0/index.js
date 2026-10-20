import { getNamespace } from "./provider";
import other from "./other";

it("invalidates cached namespace code when a conservative consumer is added or removed", () => {
	const ns = getNamespace();
	const otherNs = other();
	expect(ns.LONG_EXPORT).toBe("value");
	expect(otherNs.LONG_EXPORT).toBe("value");
	expect(Object.keys(otherNs).sort()).toEqual(["LONG_EXPORT", "canMangle"]);
	expect(Object.keys(ns).sort()).toEqual(["LONG_EXPORT", "canMangle"]);
	expect(ns.canMangle).toBe(WATCH_STEP !== "1");
});
