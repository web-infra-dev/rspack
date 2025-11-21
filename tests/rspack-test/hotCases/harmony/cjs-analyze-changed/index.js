import value from "./reexport";

it("should generate code correctly when outgoing module changes its exports type", async () => {
	expect(value.default).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(1);
});

module.hot.accept("./reexport");
