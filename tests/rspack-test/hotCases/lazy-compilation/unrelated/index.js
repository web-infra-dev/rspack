import value from "./module";

const neverCalled = () => import("./lazy");

it("should compile to lazy imported module", async () => {
	let generation = 0;
	module.hot.accept("./module", () => {
		generation++;
	});
	expect(value).toBe(42);
	expect(generation).toBe(0);
	await NEXT_HMR();
	expect(value).toBe(43);
	expect(generation).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(44);
	expect(generation).toBe(2);
});
