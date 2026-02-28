import "./loader.js!./a.js";
import index from "./file";

it("should rebuild abnormal module success", async () => {
	expect(index).toBe(1);
	await NEXT_HMR().catch(err => {
		expect(err.message).toMatch(/Expression expected/);
	});
	expect(index).toBe(1);
	await NEXT_HMR().catch(err => {
		expect(err.message).not.toMatch(/Expression expected/);
	});
	expect(index).toBe(1);
});

module.hot.accept();