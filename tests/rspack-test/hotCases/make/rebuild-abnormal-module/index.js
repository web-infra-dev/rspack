import "./loader.mjs!./a.js";
import index from "./file";

it("should rebuild abnormal module success", async () => {
	expect(index).toBe(1);
	await NEXT_HMR().catch(err => {
		expect(err.message).toMatch(/Expression expected/);
	});
	expect(index).toBe(1);
	// Compilation recovers, but the failed update leaves the HMR runtime non-idle.
	await expect(NEXT_HMR()).rejects.toThrow("check() is only allowed in idle status");
	expect(index).toBe(1);
});

module.hot.accept();
