import value from "./file";

it("should isolated module entry2 work", async () => {
	expect(COMPILER_INDEX).toBe(1);
	expect(value).toBe(3);
	await NEXT_HMR();
	expect(value).toBe(4);
});

module.hot.accept("./file");