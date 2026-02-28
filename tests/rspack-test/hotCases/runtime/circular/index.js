import a from "./a";

it("should not throw on circular dependencies", async () => {
	expect(a).toBe(1);
	await NEXT_HMR();
	expect(a).toBe(2);
});

module.hot.accept("./a");