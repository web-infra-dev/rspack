import a from "./a";

it("should abort when module is not accepted", async () => {
	expect(a).toBe(1);
	await NEXT_HMR({ ignoreErrored: true });
	expect(a).toBe(1);
	await NEXT_HMR({ ignoreErrored: true });
	expect(a).toBe(3);
});

module.hot.accept("./a");
