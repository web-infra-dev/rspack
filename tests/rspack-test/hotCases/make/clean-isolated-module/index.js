import value from "./a";

it("should make clean isolated module works", async () => {
	expect(value).toBe("cba");
	await NEXT_HMR();
	expect(value).toBe("a");
});

module.hot.accept("./a");
