import { value } from "./reexport";

it("should auto-reexport an ES6 imported value on accept with newTreeshaking", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(2);
});

module.hot.accept("./reexport");

