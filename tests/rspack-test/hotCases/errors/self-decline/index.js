import a from "./a";

it("should abort when module is declined by itself", async () => {
	expect(a).toBe(1);
	try {
		await NEXT_HMR();
	} catch (err) {
		expect(err.message).toMatch(/Aborted because of self decline: \.\/a\.js/);
		expect(err.message).toMatch(/Update propagation: \.\/c\.js -> \.\/b\.js -> \.\/a\.js/);
	}
});

module.hot.accept("./a");
