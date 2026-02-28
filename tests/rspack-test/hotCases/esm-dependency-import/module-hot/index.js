import { val } from "./module";

it("should fail accept changes", async () => {
	expect(val).toBe(1);
	try {
		await NEXT_HMR();
	} catch (err) {
		expect(err.message).toMatch(/Aborted because \.\/node_modules\/dep1\/file.js is not accepted/);
		expect(err.message).toMatch(/Update propagation: \.\/node_modules\/dep1\/file.js -> \.\/node_modules\/dep1\/exports\.js -> \.\/module\.js -> \.\/index\.js/);
	}
});
