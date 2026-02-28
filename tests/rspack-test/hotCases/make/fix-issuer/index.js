import num from "./file";

it("should clean isolated cycle", async () => {
	expect(num).toBe(1);
	await NEXT_HMR();
	expect(num).toBe(2);
});

module.hot.accept("./file");
