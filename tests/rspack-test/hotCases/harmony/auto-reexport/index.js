import { value } from "./reexport";

it("should auto-reexport an ES6 imported value on accept", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(2);
});