import num from "./file";

it("should clean isolated cycle", async () => {
	expect(num).toBe(1);
	await NEXT_HMR();
	expect(num).toBe(2);
	await NEXT_HMR();
	expect(num).toBe(3);
	await NEXT_HMR();
	expect(num).toBe(4);
});

module.hot.accept("./file");