import { getEnums, getAgain, spread, read, missing, deleteMissing } from "./provider";
import { direct, sum, readCurrent } from "./consumer";
import { enumsNs } from "./reexport";
import * as cjs from "./cjs";
import * as bailout from "./bailout";
import * as dynamic from "./dynamic";
import * as unknown from "./unknown";

it("keeps exports mangleable when a namespace escapes", () => {
	const ns = getEnums();
	expect(ns.ENUM_A).toBe("a-value");
	expect(ns.ENUM_B).toBe("b-value");
	expect(ns.NUM).toBe(42);
	expect(ns.default).toBe("default-value");
	expect(ns["not-an-identifier"]).toBe("unusual");
	expect(ns.canMangle).toBe(EXPECT_MANGLE);
	expect(direct).toBe("a-value");
	expect(sum).toBe(43);
	expect(getAgain()).toBe(ns);
	expect(read()).toEqual(["b-value", 42]);
});

it("enumerates original names and preserves live bindings", () => {
	const ns = getEnums();
	expect(Object.keys(ns).sort()).toEqual([
		"ENUM_A", "ENUM_B", "NUM", "canMangle", "currentValue", "default",
		"increment", "not-an-identifier"
	].sort());
	expect(ns.__esModule).toBe(true);
	expect(Object.prototype.toString.call(ns)).toBe("[object Module]");
	expect(spread().currentValue).toBe(0);
	ns.increment();
	expect(ns.currentValue).toBe(1);
	expect(readCurrent()).toBe(1);
	expect(spread().currentValue).toBe(1);
});

it("preserves missing members and non-configurable export getters", () => {
	expect(missing()).toBeUndefined();
	expect(deleteMissing()).toEqual([true, true]);
	const ns = getEnums();
	expect(Object.getOwnPropertyDescriptor(ns, "NUM").configurable).toBe(false);
	expect(() => { delete ns.NUM; }).toThrow(TypeError);
});

it("preserves escaping namespace re-exports and CommonJS interop", () => {
	const get = () => enumsNs;
	expect(get().ENUM_A).toBe("a-value");
	expect(get().default).toBe("default-value");
	const getCjs = () => cjs;
	expect(getCjs().CJS_A).toBe("cjs-a");
	expect(getCjs().CJS_B).toBe("cjs-b");
});

it("lets ordinary whole-object references override mangleable escapes", async () => {
	const escaped = bailout;
	const required = require("./bailout");
	expect(escaped.BAILOUT_A).toBe("bailout-a");
	expect(required.BAILOUT_A).toBe("bailout-a");
	expect(Object.keys(required).sort()).toEqual(["BAILOUT_A", "canMangle"]);
	expect(escaped).toBe(required);
	expect(escaped.canMangle).toBe(false);
	const staticallyImported = dynamic;
	const imported = await import("./dynamic");
	expect(imported.DYN_A).toBe("dyn-a");
	expect(staticallyImported.DYN_B).toBe("dyn-b");
	expect(imported.canMangle).toBe(false);
	expect(Object.keys(imported).sort()).toEqual(["DYN_A", "DYN_B", "canMangle"]);
	expect(staticallyImported).toBe(imported);
});

it("keeps unknown export names available", () => {
	const ns = unknown;
	expect(ns.known).toBe("known");
	expect(ns.UNKNOWN_A).toBe("unknown-a");
	expect(ns.canMangle).toBe(false);
});
