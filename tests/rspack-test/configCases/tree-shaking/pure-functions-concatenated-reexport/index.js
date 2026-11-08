import { Item } from "./lib";

const fs = require("fs");
const moduleEffects = globalThis.__concatenatedPureModuleEffects;
const impureCalls = globalThis.__concatenatedImpureCalls;
const baseEffects = globalThis.__concatenatedPureBaseEffects || 0;
delete globalThis.__concatenatedPureModuleEffects;
delete globalThis.__concatenatedImpureCalls;
delete globalThis.__concatenatedPureBaseEffects;

it("should preserve deferred purity checks after concatenating a re-export target", () => {
	expect(typeof Item).toBe("function");
	expect(moduleEffects).toBe(1);
	expect(impureCalls).toBe(1);
	expect(baseEffects).toBe(KEEP_BASE ? 1 : 0);

	if (!KEEP_BASE) {
		const source = fs.readFileSync(__filename, "utf-8");
		const marker = ["UNUSED", "CONCATENATED", "PURE", "ARGUMENT"].join("_");
		expect(source).not.toContain(marker);
	}
});
