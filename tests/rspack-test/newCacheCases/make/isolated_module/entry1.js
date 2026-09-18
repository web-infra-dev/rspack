import value from "./file";

it("should isolated module entry1 work", async () => {
	expect(COMPILER_INDEX).toBe(0);
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(2);
	await NEXT_START();
});

module.hot.accept("./file");