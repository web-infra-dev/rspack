it("should preserve filesystem cache after module resolution failures", async () => {
	// The first build only supplies the test that drives subsequent compilers.
	await expect(NEXT_START()).rejects.toThrow("Can't resolve './missing.js'");
	await expect(NEXT_START()).rejects.toThrow("Can't resolve './missing.js'");
	// With bail disabled, the same failure is reported in compilation stats.
	const stats = await NEXT_START();
	expect(stats.errors).toHaveLength(1);
});
---
import "./missing.js";
---
import "./missing.js";
---
it("should report the missing module without bail and recover", async () => {
	expect(() => require("./missing.js")).toThrow("Cannot find module './missing.js'");
	await NEXT_START();
});
---
it("should build successfully after fixing the unresolved import", async () => {
	expect(require("./value.js")).toBe("recovered");
	if (COMPILER_INDEX === 4) await NEXT_START();
});
