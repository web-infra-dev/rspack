import value from "./file";

it("should accept a dependencies and require a new value", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(2);
	await NEXT_HMR();
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(3);
});

module.hot.accept("./file");