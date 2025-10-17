import value from "./a";

it("should run module.hot.accept(…)", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(2);
});

module.hot.accept("./a");
