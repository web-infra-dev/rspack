import { getEnums, read } from "./provider";
import { ENUM_A as direct } from "../mangle-escaping-namespace/enums";
import * as external from "../mangle-escaping-namespace-external/provider";

it("preserves escaping namespaces in modern-module output", () => {
	const ns = getEnums();
	expect(ns.ENUM_A).toBe("a-value");
	expect(ns.NUM).toBe(42);
	expect(ns.default).toBe("default-value");
	expect(direct).toBe("a-value");
	expect(read()).toEqual(["b-value", 42]);
	expect(getEnums()).toBe(ns);
	ns.increment();
	expect(ns.currentValue).toBe(1);
});

it("preserves escaping namespaces for modules outside the hoisted group", () => {
	const ns = external;
	expect(ns.e00).toBe("v00");
	expect(ns.e59).toBe("v59");
	expect(Object.keys(ns)).toHaveLength(61);
});
