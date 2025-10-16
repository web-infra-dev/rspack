import a from "./a";
import b from "./b";

it("should abort when module is not accepted", async () => {
	expect(a).toBe(2);
	expect(b).toBe(1);
	try {
		await NEXT_HMR();
	} catch (err) {
		expect(err.message).toMatch(/Aborted because \.\/c\.js is not accepted/);
		expect(err.message).toMatch(/Update propagation: \.\/c\.js -> \.\/b\.js -> \.\/index\.js/);
	}
});
